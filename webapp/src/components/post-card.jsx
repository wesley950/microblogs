import { Link } from "react-router-dom";
import UserAvatar from "./user-avatar";
import Interactions from "./interactions";
import MediaCarousel from "./media-carousel";

const URL_REGEX = /https?:\/\/[^\s]+/g;

function parseBody(body) {
  let mediaUrls = body.match(URL_REGEX) || [];
  return {
    paragraphs: body.split("\n").map((paragraph) => paragraph.trim()),
    mediaUrls,
  };
}

export default function PostCard({
  post,
  truncate = false,
  linkToPost = false,
}) {
  const { paragraphs, mediaUrls } = parseBody(post.body);

  return (
    <div className="card">
      <div className="card-body">
        <div className="position-relative vstack gap-2">
          <Link
            to={`/post/${post.id}`}
            className={`text-decoration-none ${
              linkToPost ? "stretched-link" : ""
            }`}
          >
            <UserAvatar
              username={post.user.username}
              realName={post.user.realName}
            />
          </Link>

          {truncate ? (
            <p
              className="text-wrap overflow-hidden"
              style={{
                maxHeight: "100px",
              }}
            >
              {post.body}
            </p>
          ) : (
            paragraphs.map((paragraph, index) => (
              <p key={`post-${post.id}-paragraph-${index}`}>{paragraph}</p>
            ))
          )}

          {mediaUrls.length > 0 && (
            <MediaCarousel
              postId={post.id}
              mediaUrls={mediaUrls}
              maxImageHeight={"600px"}
            />
          )}
        </div>

        <Interactions
          postId={post.id}
          likedByUser={post.likedByMe}
          likeCount={post.likeCount}
          replyCount={post.replyCount}
          shareable
        />
      </div>
    </div>
  );
}
