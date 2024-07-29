use std::{fs::create_dir, path::Path};

use actix_files::NamedFile;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{
    get,
    http::header::{ContentDisposition, DispositionType},
    post,
    web::{self, ServiceConfig},
    HttpResponse,
};
use chrono::NaiveDateTime;
use diesel::{
    query_dsl::methods::FilterDsl, Connection, ExpressionMethods, Insertable, Queryable,
    RunQueryDsl, Selectable, SelectableHelper,
};
use microblogs::{errors::ServiceError, generate_uid, schema, AppState, DbPool};
use serde::Serialize;

use crate::users::UserDetails;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    files: Vec<TempFile>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::attachments)]
struct NewAttachment {
    pub uploader_id: i32,
    pub uuid: String,
    pub file_name: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::attachments)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct Attachment {
    pub id: i32,
    pub uploader_id: i32,
    pub uuid: String,
    pub file_name: String,
    pub uploaded_at: NaiveDateTime,
    pub deleted: bool,
}

#[derive(Serialize)]
struct AttachmentRead {
    id: i32,
    uploader_id: i32,
    uuid: String,
    file_name: String,
    uploaded_at: String,
    deleted: bool,
}

impl From<Attachment> for AttachmentRead {
    fn from(attachment: Attachment) -> Self {
        AttachmentRead {
            id: attachment.id,
            uploader_id: attachment.uploader_id,
            uuid: attachment.uuid,
            file_name: attachment.file_name,
            uploaded_at: attachment.uploaded_at.to_string(),
            deleted: attachment.deleted,
        }
    }
}

#[post("/upload")]
async fn upload_attachment(
    MultipartForm(form): MultipartForm<UploadForm>,
    app_state: web::Data<AppState>,
    pool: web::Data<DbPool>,
    current_user: UserDetails,
) -> Result<HttpResponse, actix_web::Error> {
    use schema::attachments::dsl::*;

    let mut attachments_to_save: Vec<NewAttachment> = Vec::new();

    for file in form.files {
        match file.content_type {
            Some(content_type) => {
                // only allow images and videos
                match content_type.type_().as_str() {
                    "image" | "video" => {}
                    _ => return Err(ServiceError::BadRequest(format!("Somente imagens e vídeos são permitidos. Um dos arquivos tem o seguinte tipo: {}.", content_type)).into()),
                }
            }
            None => {
                return Err(ServiceError::BadRequest(format!(
                    "O cabeçalho \"Content-Type\" não está presente."
                ))
                .into())
            }
        }

        let fname = match file.file_name {
            Some(fname) => fname.to_string(),
            None => {
                return Err(ServiceError::BadRequest(format!(
                    "O cabeçalho \"Content-Disposition\" não está presente."
                ))
                .into())
            }
        };
        let fpath = Path::new(&fname);

        let stem = match fpath.file_stem() {
            Some(stem) => stem.to_str(),
            None => {
                return Err(ServiceError::BadRequest(format!(
                    "Falha ao extrair o nome do arquivo \"{}\".",
                    fname
                ))
                .into())
            }
        };
        let stem = match stem {
            Some(stem) => stem,
            None => {
                return Err(ServiceError::InternalServerError(format!(
                    "Falha ao converter o nome do arquivo \"{}\".",
                    fname
                ))
                .into())
            }
        };
        let extension = match fpath.extension() {
            Some(extension) => extension,
            None => {
                return Err(ServiceError::BadRequest(format!(
                    "Falha ao extrair a extensão do arquivo \"{}\".",
                    fname
                ))
                .into())
            }
        };
        let extension = match extension.to_str() {
            Some(extension) => extension,
            None => {
                return Err(ServiceError::BadRequest(format!(
                    "Falha ao converter a extensão do arquivo \"{}\".",
                    fname
                ))
                .into())
            }
        };

        let attachment_uuid = generate_uid();
        let saved_file_name = format!("{}.{}", stem, extension);
        let path = format!(
            "{}/{}/{}",
            app_state.uploads_dir, attachment_uuid, saved_file_name
        );

        if let Err(_) = create_dir(format!("{}/{}", app_state.uploads_dir, attachment_uuid)) {
            return Err(ServiceError::InternalServerError(format!(
                "Não foi possível criar o diretório para o arquivo \"{}\".",
                fname
            ))
            .into());
        };

        match file.file.persist(path) {
            Ok(_) => {
                let new_attachment = NewAttachment {
                    uploader_id: current_user.id,
                    uuid: attachment_uuid.to_string(),
                    file_name: saved_file_name,
                };

                attachments_to_save.push(new_attachment);
            }
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Não foi possível salvar o arquivo \"{}\".",
                    fname
                ))
                .into())
            }
        }
    }

    let result = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Impossível conectar ao banco de dados."
                )))
            }
        };

        match conn.transaction::<Vec<AttachmentRead>, diesel::result::Error, _>(|conn| {
            let mut uploaded_attachments: Vec<AttachmentRead> = Vec::new();
            for attachment in attachments_to_save {
                match diesel::insert_into(attachments)
                    .values(&attachment)
                    .returning(Attachment::as_returning())
                    .get_result(conn)
                {
                    Ok(attachment) => uploaded_attachments.push(attachment.into()),
                    Err(_) => return Err(diesel::result::Error::RollbackTransaction),
                }
            }

            Ok(uploaded_attachments)
        }) {
            Ok(result) => Ok(result),
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Um ou mais arquivos não puderam ser adicionados ao banco de dados."
                ))
                .into())
            }
        }
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
}

#[get("/{attachment_uuid}")]
async fn download_attachment(
    attachment_uuid: web::Path<String>,
    app_state: web::Data<AppState>,
    pool: web::Data<DbPool>,
    _current_user: UserDetails,
) -> Result<NamedFile, actix_web::Error> {
    use schema::attachments::dsl::*;

    let target_attachment_uuid = attachment_uuid.clone();
    let fpath = web::block(move || {
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(_) => {
                return Err(ServiceError::InternalServerError(format!(
                    "Impossível conectar ao banco de dados."
                )))
            }
        };

        let attachment: Attachment = match attachments
            .filter(uuid.eq(attachment_uuid.as_str()))
            .first(&mut conn)
        {
            Ok(attachment) => attachment,
            Err(_) => {
                return Err(ServiceError::NotFound(format!(
                    "Anexo \"{}\" não encontrado.",
                    attachment_uuid.clone()
                )))
            }
        };

        let path = format!(
            "{}/{}/{}",
            app_state.uploads_dir, attachment.uuid, attachment.file_name
        );

        Ok(path)
    })
    .await??;

    let stream = NamedFile::open(fpath);

    match stream {
        Ok(stream) => {
            Ok(stream
                .use_last_modified(true)
                .set_content_disposition(ContentDisposition {
                    disposition: DispositionType::Inline,
                    parameters: vec![],
                }))
        }
        Err(_) => Err(ServiceError::InternalServerError(format!(
            "Não foi possível abrir o arquivo do anexo \"{}\".",
            target_attachment_uuid
        ))
        .into()),
    }
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/attachments")
            .service(upload_attachment)
            .service(download_attachment),
    );
}
