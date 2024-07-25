import { useLoaderData } from "react-router-dom";
import Feed from "../components/feed";
import axios from "axios";

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

export default function Index() {
  const { posts } = useLoaderData();

  return (
    <div
      className="container vh-100 d-flex vstack gap-2"
      style={{
        maxWidth: 400,
      }}
    >
      <h1>Publicações no Microblogs</h1>
      <Feed posts={posts} />
    </div>
  );
}
