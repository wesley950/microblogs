import axios from "axios";
import { useFetcher } from "react-router-dom";

export async function likeAction({ request, params }) {
  try {
    let response = await axios.post("/posts/like?id=" + params.postId);
    if (response.status === 200) {
      return response.data;
    }
  } catch (error) {
    console.log(error);
  }
  return null;
}

export async function unlikeAction({ request, params }) {
  try {
    let response = await axios.delete("/posts/like?id=" + params.postId);
    if (response.status === 200) {
      return response.data;
    }
  } catch (error) {
    console.log(error);
  }
  return null;
}

export default function Interactions({
  postId,
  likedByUser,
  likeCount,
  replyCount,
}) {
  const fetcher = useFetcher();

  return (
    <div className="hstack gap-2 justify-content-evenly text-center">
      <fetcher.Form
        method="post"
        action={likedByUser ? `/post/${postId}/unlike` : `/post/${postId}/like`}
      >
        <button className="btn link-danger btn-md">
          {likedByUser ? (
            <i className="bi bi-heart-fill"></i>
          ) : (
            <i className="bi bi-heart"></i>
          )}{" "}
          {likeCount} curtidas
        </button>
      </fetcher.Form>
      <button className="btn link-primary btn-md">
        <i className="bi bi-chat"></i> {replyCount} respostas
      </button>
      <button className="btn link-info btn-md">
        <i className="bi bi-share"></i> Compartilhar
      </button>
    </div>
  );
}
