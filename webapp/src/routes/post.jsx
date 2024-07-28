import {
  useFetcher,
  useNavigate,
  useNavigation,
  useParams,
} from "react-router-dom";
import PostCard from "../components/post-card";
import axios from "axios";
import { useEffect, useRef, useState } from "react";
import PostBodyTextarea from "../components/post-body-textarea";

const PAGE_SIZE = 5;

async function loadPost(postUuid, repliesOffset, repliesLimit) {
  try {
    let response = await axios.get(
      `/feeds/replies?uuid=${postUuid}&offset=${repliesOffset}&limit=${repliesLimit}`
    );
    if (response.status === 200) {
      let parentPost = response.data.parent;
      let postReplies = response.data.replies;

      return {
        uuid: parentPost.uuid,
        body: parentPost.body,
        createdAt: parentPost.created_at,
        likeCount: parentPost.like_count,
        replyCount: parentPost.reply_count,
        likedByMe: parentPost.liked_by_user,
        user: {
          username: parentPost.poster.username,
          realName: parentPost.poster.real_name,
        },
        replies: postReplies.map((reply) => {
          return {
            uuid: reply.uuid,
            body: reply.body,
            likeCount: reply.like_count,
            replyCount: reply.reply_count,
            likedByMe: reply.liked_by_user,
            user: {
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

  return null;
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
  const [post, setPost] = useState(null);
  const fetcher = useFetcher();
  const { postUuid } = useParams();
  const navigate = useNavigate();
  const navigation = useNavigation();
  const formRef = useRef(null);

  useEffect(() => {
    loadPost(postUuid, 0, PAGE_SIZE).then((newPost) => {
      setPost(newPost);
    });
  }, [postUuid]);

  useEffect(() => {
    if (fetcher.data) {
      let fetchReplyData = async () => {
        let replyData = await loadPost(fetcher.data.newReply.uuid, 0, 0);
        setPost((post) => {
          return {
            ...post,
            replies: [replyData, ...post.replies],
          };
        });
      };

      fetchReplyData();
    }
  }, [fetcher.data]);

  useEffect(() => {
    let handler = () => {
      if (
        window.scrollY / (document.body.scrollHeight - window.innerHeight) >
        0.8
      ) {
        let reloadPost = async () => {
          let newPost = await loadPost(
            postUuid,
            post.replies.length,
            PAGE_SIZE
          );
          setPost((post) => ({
            ...post,
            replies: post.replies.concat(newPost.replies),
          }));
        };
        reloadPost();
      }
    };
    window.addEventListener("scrollend", handler);

    return () => window.removeEventListener("scrollend", handler);
  }, [post?.replies]);

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

          {post.replies.map((reply, replyIndex) => {
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
