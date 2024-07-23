import { Form, useLoaderData, useNavigate } from "react-router-dom";
import UserAvatar from "../components/user-avatar";
import Interactions from "../components/interactions";
import ImageCarousel from "../components/image-carousel";
import PostCard from "../components/post-card";

export async function loader({ params }) {
  let userId = Math.floor(Math.random() * 100);

  let replies = [];

  for (let i = 0; i < 10; i++) {
    let replierId = Math.floor(Math.random() * 1000);
    replies.push({
      id: Math.floor(Math.random() * 1000),
      body: `The standard chunk of Lorem Ipsum used since the 1500s is reproduced below for those interested. Sections 1.10.32 and 1.10.33 from "de Finibus Bonorum et Malorum" by Cicero are also reproduced in their exact original form, accompanied by English versions from the 1914 translation by H. Rackham.`,
      likeCount: Math.floor(Math.random() * 1000),
      replyCount: Math.floor(Math.random() * 1000),
      imageUrls: [
        `https://picsum.photos/seed/2${replierId}/400/500`,
        `https://picsum.photos/seed/3${replierId}/500/400`,
      ],
      user: {
        id: replierId,
        username: `commentador${replierId}`,
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
      likeCount: Math.floor(Math.random() * 1000),
      replyCount: Math.floor(Math.random() * 1000),
      replies,
      user: {
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

      <PostCard post={post} />

      <hr />
      <Form className="vstack gap-1">
        <textarea className="form-control" placeholder="Escreva uma resposta" />
        <div className="hstack d-flex justify-content-end">
          <button type="submit" className="btn btn-primary">
            <i className="bi bi-chat"></i> Responder
          </button>
        </div>
      </Form>
      <hr />

      {post.replies.map((reply, replyIndex) => {
        return (
          <PostCard key={`post-${post.id}-reply-${replyIndex}`} post={reply} />
        );
      })}
    </div>
  );
}
