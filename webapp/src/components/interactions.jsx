import axios from "axios";
import { useEffect, useState } from "react";
import { useFetcher } from "react-router-dom";

export async function likeAction({ request, params }) {
  try {
    let formData = await request.formData();
    let response = null;
    if (formData.get("like") === "true") {
      response = await axios.post("/posts/like?id=" + params.postId);
    } else {
      response = await axios.delete("/posts/like?id=" + params.postId);
    }
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
  const [liked, setLiked] = useState(likedByUser);
  const [newLikeCount, setNewLikeCount] = useState(likeCount);

  useEffect(() => {
    if (fetcher.formData) {
      let liked = fetcher.formData.get("like") === "true";
      setLiked(liked);
      setNewLikeCount(newLikeCount => liked ? newLikeCount + 1 : newLikeCount - 1);
    }
  }, [fetcher.formData]);

  return (
    <div className="hstack gap-2 justify-content-evenly text-center">
      <fetcher.Form method="post" action={`/post/${postId}/like`}>
        <button
          className="btn link-danger btn-md"
          name="like"
          value={liked ? "false" : "true"}
        >
          <i className={liked ? "bi bi-heart-fill" : "bi bi-heart"} />{" "}
          {newLikeCount}
          {newLikeCount === 1 ? " curtida" : " curtidas"}
        </button>
      </fetcher.Form>
      <button className="btn link-primary btn-md">
        <i className="bi bi-chat"></i> {replyCount} respostas
      </button>
      <button className="btn link-info btn-md">
        <i className="bi bi-share"></i> compartilhar
      </button>
    </div>
  );
}
