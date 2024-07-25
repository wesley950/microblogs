import { Form, useLoaderData, useNavigate } from "react-router-dom";
import UserAvatar from "../components/user-avatar";
import Interactions from "../components/interactions";
import ImageCarousel from "../components/image-carousel";
import PostCard from "../components/post-card";
import axios from "axios";

export async function loader({ params }) {
  let post = null;

  try {
    let response = await axios.get(
      `/feeds/replies?id=${params.postId}&offset=0&limit=20`
    );
    if (response.status === 200) {
      let parentPost = response.data.parent;
      let postReplies = response.data.replies;

      post = {
        id: parentPost.id,
        body: parentPost.body,
        createdAt: parentPost.created_at,
        likeCount: parentPost.like_count,
        replyCount: parentPost.reply_count,
        imageUrls: [],
        likedByMe: parentPost.liked_by_user,
        user: {
          id: parentPost.poster.id,
          username: parentPost.poster.username,
          realName: parentPost.poster.real_name,
        },
        replies: postReplies.map((reply) => {
          return {
            id: reply.id,
            body: reply.body,
            likeCount: reply.like_count,
            replyCount: reply.reply_count,
            imageUrls: [],
            likedByMe: reply.liked_by_user,
      user: {
              id: reply.poster.id,
              username: reply.poster.username,
              realName: reply.poster.real_name,
      },
          };
        }),
      };
    }
  } catch (error) {
    console.log(error);
  }

  return {
    post,
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
