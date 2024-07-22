import { useLoaderData, useNavigate } from "react-router-dom";
import UserAvatar from "./user-avatar";
import Interactions from "./interactions";
import ImageCarousel from "./image-carousel";

export async function loader({ params }) {
  let userId = Math.floor(Math.random() * 100);

  let comments = [];

  for (let i = 0; i < 10; i++) {
    let commenterId = Math.floor(Math.random() * 1000);
    comments.push({
      id: Math.floor(Math.random() * 1000),
      text: `The standard chunk of Lorem Ipsum used since the 1500s is reproduced below for those interested. Sections 1.10.32 and 1.10.33 from "de Finibus Bonorum et Malorum" by Cicero are also reproduced in their exact original form, accompanied by English versions from the 1914 translation by H. Rackham.`,
      likes: Math.floor(Math.random() * 1000),
      poster: {
        id: commenterId,
        username: `commentador${commenterId}`,
        realName: "Nome Real",
      },
    });
  }

  return {
    post: {
      id: params.postId,
      body: `Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.`,
      imageUrls: [
        `https://picsum.photos/seed/1${params.postId}/400/400`,
        `https://picsum.photos/seed/2${params.postId}/400/500`,
        `https://picsum.photos/seed/3${params.postId}/500/400`,
      ],
      createdAt: new Date(),
      likes: Math.floor(Math.random() * 1000),
      commentCount: Math.floor(Math.random() * 1000),
      comments,
      poster: {
        id: userId,
        username: `user${userId}`,
        realName: "Nome Real",
      },
    },
  };
}

export async function action({ request, params }) {}

export default function Post() {
  const { post } = useLoaderData();
  const navigate = useNavigate();

  return (
    <div
      className="container vh-100 d-flex vstack gap-2 my-2"
      style={{
        maxWidth: 600,
      }}
    >
      <div className="hstack">
        <button className="btn link-primary" onClick={() => navigate(-1)}>
          <i className="bi bi-arrow-left"></i> Voltar
        </button>
      </div>
      <UserAvatar
        username={post.poster.username}
        realName={post.poster.realName}
      />

      {post.body.split("\n").map((paragraph, index) => (
        <p key={`post-paragraph-${index}`}>{paragraph}</p>
      ))}

      <ImageCarousel
        postId={post.id}
        imageUrls={post.imageUrls}
        maxImageHeight={"1000px"}
      />

      <Interactions likes={post.likes} shareable />

      <h3>Comentários ({post.commentCount})</h3>
      {post.comments.map((comment, commendIndex) => (
        <div
          key={`post-${post.id}-comment-${commendIndex}`}
          className="vstack gap-2"
        >
          <UserAvatar
            username={comment.poster.username}
            realName={comment.poster.realName}
          />
          <p>{comment.text}</p>
          <Interactions likes={comment.likes} />
        </div>
      ))}
    </div>
  );
}
