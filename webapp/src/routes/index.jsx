import { Form, redirect, useLoaderData } from "react-router-dom";
import Feed from "../components/feed";
import axios from "axios";
import AutoResizableTextarea from "../components/auto-resizable-textarea";

export async function loader() {
  let posts = [];

  try {
    let response = await axios.get("/feeds/?offset=0&limit=10");
    if (response.status === 200) {
      posts = response.data.posts.map((post) => {
        return {
          id: post.id,
          body: post.body,
          imageUrls: [],
          createdAt: post.created_at,
          likeCount: post.like_count,
          replyCount: post.reply_count,
          likedByMe: post.liked_by_user,
          user: {
            id: post.poster.id,
            username: post.poster.username,
            realName: post.poster.real_name,
          },
        };
      });
    }
  } catch (error) {
    console.log(error);
  }

  return {
    posts,
  };
}

export async function action({ request }) {
  const formData = await request.formData();
  const data = JSON.stringify({
    body: formData.get("body"),
  });

  try {
    let response = await axios.post("/posts/create", data);
    if (response.status === 200) {
      return redirect(`/post/${response.data.id}`);
    }
  } catch (error) {
    console.log(error);
  }

  return null;
}

export default function Index() {
  const { posts } = useLoaderData();

  return (
    <div
      className="container vh-100 d-flex vstack gap-2 my-2"
      style={{
        maxWidth: 400,
      }}
    >
      <Form method="post" className="vstack gap-1">
        <div className="form-floating">
          <AutoResizableTextarea
            className="form-control"
            name="body"
            placeholder=""
          />
          <label>faça uma publicação...</label>
        </div>
        <button className="btn btn-primary" type="submit">
          <i className="bi bi-pencil"></i> publicar
        </button>
      </Form>
      <Feed posts={posts} />
    </div>
  );
}
