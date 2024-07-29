import { useEffect, useRef, useState } from "react";
import { parseBody } from "../utils/media";
import axios from "axios";

export default function PostBodyTextarea({ placeholder, name = "body" }) {
  const [previewMediaUrls, setPreviewMediaUrls] = useState([]);
  const textareaRef = useRef(null);
  const dropZoneRef = useRef(null);

  useEffect(() => {
    if (textareaRef.current && textareaRef.current?.value === "") {
      setPreviewMediaUrls([]);
    }
  }, [textareaRef.current, textareaRef.current?.value]);

  useEffect(() => {
    let dropZone = dropZoneRef.current;
    if (dropZone) {
      let dragOverHandler = (e) => {
        e.preventDefault();
        e.stopPropagation();
        dropZone.style.backgroundColor = "#e1e1e1";
      };
      let dragLeaveHandler = (e) => {
        dropZone.style.backgroundColor = "transparent";
      };
      let dropHandler = (e) => {
        e.preventDefault();
        e.stopPropagation();
        dropZone.style.backgroundColor = "transparent";

        let uploadFiles = async () => {
          try {
            let formData = new FormData();
            for (let i = 0; i < e.dataTransfer.files.length; i++) {
              formData.append("files", e.dataTransfer.files[i]);
            }
            let response = await axios.post("/attachments/upload", formData, {
              headers: {
                "Content-Type": "multipart/form-data",
              },
            });
            if (response.status === 200) {
              response.data.forEach((attachment) => {
                textareaRef.current.value += `\n${
                  import.meta.env.VITE_API_BASE_ADDRESS
                }/attachments/${attachment.uuid}`;
              });

              let { mediaUrls } = parseBody(textareaRef.current.value);
              setPreviewMediaUrls(mediaUrls);
            }
          } catch (error) {
            console.log(error);
          }
        };
        uploadFiles();
      };

      dropZone.addEventListener("dragover", dragOverHandler);
      dropZone.addEventListener("dragleave", dragLeaveHandler);
      dropZone.addEventListener("drop", dropHandler);

      return () => {
        dropZone.removeEventListener("dragover", dragOverHandler);
        dropZone.removeEventListener("dragleave", dragLeaveHandler);
        dropZone.removeEventListener("drop", dropHandler);
      };
    }
  }, [dropZoneRef.current]);

  return (
    <>
      <div className="form-floating" ref={dropZoneRef}>
        <textarea
          className="form-control"
          name={name}
          ref={textareaRef}
          placeholder=""
          onInput={(e) => {
            const text = e.target.value || "";
            const { mediaUrls } = parseBody(text);
            setPreviewMediaUrls(mediaUrls);
          }}
          style={{ height: "150px" }}
        />
        <label>{placeholder}</label>
      </div>
      <div className="hstack gap-1 d-flex flex-row flex-wrap">
        {previewMediaUrls.map((mediaUrl, index) => (
          <img
            src={mediaUrl}
            key={`post-form-media-preview-${index}`}
            width={80}
            className="img-thumbnail img-fluid"
          />
        ))}
      </div>
    </>
  );
}
