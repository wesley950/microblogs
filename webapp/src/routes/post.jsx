import {
  useFetcher,
  useLoaderData,
  useNavigate,
  useNavigation,
} from "react-router-dom";
import PostCard from "../components/post-card";
import axios from "axios";
import AutoResizableTextarea from "../components/auto-resizable-textarea";
import { useEffect, useRef } from "react";

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

export async function action({ request, params }) {
  let formData = await request.formData();
  let data = JSON.stringify({
    parent_id: parseInt(params.postId),
    body: formData.get("reply"),
  });

  try {
    let response = await axios.post(`/posts/create`, data);
    if (response.status === 200) {
      let newReply = response.data;
      return { newReply };
    }
  } catch (error) {
    console.log(error);
  }

  return null;
}

export default function Post() {
  const { post } = useLoaderData();
  const fetcher = useFetcher();
  const navigate = useNavigate();
  const navigation = useNavigation();
  const formRef = useRef(null);

  useEffect(() => {
    if (
      navigation.state === "idle" &&
      fetcher.state === "idle" &&
      formRef.current
    ) {
      formRef.current.reset();
    }
  }, [navigation.state, fetcher.state]);

  return (
    <div
      className="container vh-100 d-flex vstack gap-2 my-2"
      style={{
        maxWidth: 600,
      }}
    >
      <div className="hstack">
        <button className="btn link-primary" onClick={() => navigate(-1)}>
          <i className="bi bi-arrow-left"></i> voltar
        </button>
      </div>

      <PostCard post={post} />

      <hr />
      <fetcher.Form method="post" className="vstack gap-1" ref={formRef}>
        <div className="form-floating">
          <input
            type="number"
            name="parentId"
            value={post.id}
            readOnly
            hidden
          />
          <AutoResizableTextarea
            className="form-control"
            name="reply"
            defaultValue=""
            placeholder=""
          />
          <label>escreva uma resposta...</label>
        </div>
        <div className="hstack d-flex justify-content-end">
          <button type="submit" className="btn btn-primary">
            <i className="bi bi-chat"></i> responder
          </button>
        </div>
      </fetcher.Form>
      <hr />

      {post.replies.map((reply, replyIndex) => {
        return (
          <PostCard key={`post-${post.id}-reply-${replyIndex}`} post={reply} />
        );
      })}
    </div>
  );
}
