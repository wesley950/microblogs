import {
  useFetcher,
  useLoaderData,
  useNavigate,
  useNavigation,
  useParams,
} from "react-router-dom";
import PostCard from "../components/post-card";
import axios from "axios";
import { useEffect, useRef, useState } from "react";
import PostBodyTextarea from "../components/post-body-textarea";
import { parsePost } from "../utils/media";

async function loadPost(postUuid) {
  try {
    let response = await axios.get(`/feeds/details/${postUuid}`);
    if (response.status === 200) {
      return parsePost(response.data);
    }
  } catch (error) {
    console.log(error);
  }

  return null;
}

async function loadReplies(postUuid, offset = 0, limit = 5) {
  try {
    let response = await axios.get(
      `/feeds/replies/${postUuid}?offset=${offset}&limit=${limit}`
    );
    if (response.status === 200) {
      let replies = response.data.replies.map((reply) => parsePost(reply));
      return replies;
    }
  } catch (error) {
    console.log(error);
  }

  return [];
}

export async function loader({ params }) {
  let postUuid = params.postUuid;
  let post = await loadPost(postUuid);
  return post;
}

export async function action({ request, params }) {
  let formData = await request.formData();
  let data = JSON.stringify({
    parent_uuid: params.postUuid,
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
  const post = useLoaderData();
  const fetcher = useFetcher();
  const navigation = useNavigation();
  const navigate = useNavigate();
  const formRef = useRef(null);

  const [replies, setReplies] = useState([]);

  useEffect(() => {
    let fetchReplies = async () => {
      let newReplies = await loadReplies(post.uuid);
      setReplies(newReplies);
    };
    fetchReplies();
  }, [post.uuid]);

  // fetch reply data after its submitted and add at the top
  useEffect(() => {
    if (fetcher.data) {
      let fetchReplyData = async () => {
        let newReply = await loadPost(fetcher.data.newReply.uuid);
        setReplies((replies) => [newReply, ...replies]);
      };

      fetchReplyData();
    }
  }, [fetcher.data]);

  // load more replies when user scrolls
  useEffect(() => {
    let handler = () => {
      if (
        window.scrollY / (document.body.scrollHeight - window.innerHeight) >
        0.8
      ) {
        let fetchMoreReplies = async () => {
          let newReplies = await loadReplies(post.uuid, replies.length);
          setReplies((replies) => replies.concat(newReplies));
        };
        fetchMoreReplies();
      }
    };
    window.addEventListener("scrollend", handler);

    return () => window.removeEventListener("scrollend", handler);
  }, [replies]);

  // reset form after user submits
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
    <div className="container vh-100 d-flex vstack gap-2 my-2">
      <div className="hstack">
        <button className="btn link-primary" onClick={() => navigate(-1)}>
          <i className="bi bi-arrow-left"></i> voltar
        </button>
      </div>

      {post && (
        <>
          <PostCard post={post} />

          <hr />
          <fetcher.Form method="post" className="vstack gap-1" ref={formRef}>
            <div className="form-floating">
              <input
                type="number"
                name="parentId"
                value={post.uuid}
                readOnly
                hidden
              />
              <PostBodyTextarea
                id="replyTextarea"
                name="reply"
                placeholder={"escreva uma resposta..."}
              />
            </div>
            <div className="hstack d-flex justify-content-end">
              <button type="submit" className="btn btn-primary">
                <i className="bi bi-chat"></i> responder
              </button>
            </div>
          </fetcher.Form>
          <hr />

          {replies.map((reply, replyIndex) => {
            return (
              <PostCard
                key={`post-${post.uuid}-reply-${replyIndex}`}
                post={reply}
                linkToPost
              />
            );
          })}
        </>
      )}
    </div>
  );
}
