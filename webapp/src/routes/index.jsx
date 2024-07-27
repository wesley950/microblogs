import { Form, redirect, useLoaderData } from "react-router-dom";
import Feed from "../components/feed";
import axios from "axios";
import { useEffect, useState } from "react";
import PostBodyTextarea from "../components/post-body-textarea";

const PAGE_SIZE = 5;

async function loadPosts(offset, limit) {
  try {
    let response = await axios.get(`/feeds/?offset=${offset}&limit=${limit}`);
    if (response.status === 200) {
      return response.data.posts.map((post) => {
        return {
          id: post.id,
          body: post.body,
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

  return [];
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
  const [posts, setPosts] = useState([]);

  useEffect(() => {
    loadPosts(0, PAGE_SIZE).then((posts) => setPosts(posts));
  }, []);

  useEffect(() => {
    let handler = () => {
      if (
        window.scrollY / (document.body.scrollHeight - window.innerHeight) >
        0.8
      ) {
        let loadMorePosts = async () => {
          let newPosts = await loadPosts(posts.length, PAGE_SIZE);
          setPosts((posts) => posts.concat(newPosts));
        };
        loadMorePosts();
      }
    };
    window.addEventListener("scrollend", handler);

    return () => window.removeEventListener("scrollend", handler);
  }, [posts]);

  return (
    <div
      className="container vh-100 d-flex vstack gap-2 my-2"
      style={{
        maxWidth: 400,
      }}
    >
      <Form method="post" className="vstack gap-1">
        <PostBodyTextarea placeholder={"faça uma publicação..."} />
        <button className="btn btn-primary" type="submit">
          <i className="bi bi-pencil"></i> publicar
        </button>
      </Form>
      <Feed posts={posts} />
    </div>
  );
}
