import axios from "axios";
import { useEffect, useState } from "react";
import { useLoaderData } from "react-router-dom";

import PostCard from "../components/post-card";
import { parsePost } from "../utils/media";

async function loadUserPosts(username, offset = 0, limit = 5) {
  try {
    let response = await axios.get(
      `/profiles/${username}/posts?offset=${offset}&limit=${limit}`
    );
    if (response.status === 200) {
      return response.data.posts.map((post) => parsePost(post));
    }
  } catch (error) {
    console.log(error);
  }

  return [];
}

export async function loader({ params }) {
  let username = params.username;

  try {
    let response = await axios.get(`/profiles/${username}/details`);
    if (response.status === 200) {
      return {
        username: response.data.username,
        realName: response.data.real_name,
        summary: response.data.summary,
        createdAt: new Date(response.data.created_at),
      };
    }
  } catch (error) {
    console.log(error);
  }

  return null;
}

export default function Profile() {
  const profile = useLoaderData();
  const seed = encodeURI(`${profile.username}${profile.realName}`);
  const [activity, setActivity] = useState([]);

  useEffect(() => {
    async function loadActivity() {
      let newActivity = await loadUserPosts(profile.username);
      setActivity(newActivity);
    }

    loadActivity();
  }, [profile.username]);

  useEffect(() => {
    let handler = () => {
      if (
        window.scrollY / (document.body.scrollHeight - window.innerHeight) >
        0.8
      ) {
        let loadMoreActivity = async () => {
          let newActivity = await loadUserPosts(
            profile.username,
            activity.length
          );
          setActivity((activity) => activity.concat(newActivity));
        };
        loadMoreActivity();
      }
    };
    window.addEventListener("scrollend", handler);

    return () => window.removeEventListener("scrollend", handler);
  }, [activity]);

  return (
    <div className="container mt-2">
      <div className="vstack gap-2 text-center">
        <img
          src={`https://api.dicebear.com/9.x/open-peeps/svg?seed=${seed}&backgroundColor=b6e3f4,c0aede,d1d4f9,ffd5dc,ffdfbf`}
          className="mx-auto rounded-circle img-thumbnail img-fluid"
          width={120}
        />
        <h1>{profile.username}</h1>
        <h2 className="text-muted">{profile.realName}</h2>
        <p>Conta criada em {profile.createdAt.toLocaleString()}</p>
        <p>{profile.summary}</p>
      </div>

      <hr />
      <h3 className="text-center">Postagens e Respostas</h3>
      <div className="vstack gap-2">
        {activity.map((post) => {
          return (
            <PostCard
              post={post}
              key={`activity-post-card-${post.uuid}`}
              linkToPost
            />
          );
        })}
      </div>
    </div>
  );
}
