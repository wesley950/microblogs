import { Link } from "react-router-dom";
import UserAvatar from "./user-avatar";
import Interactions from "./interactions";
import MediaCarousel from "./media-carousel";
import { parseBody } from "../utils/media";

export default function PostCard({
  post,
  truncate = false,
  linkToPost = false,
}) {
  const { withoutUrls, paragraphs, mediaUrls } = parseBody(post.body);

  return (
    <div className="card">
      <div className="card-body">
        <div className="position-relative vstack gap-2">
          <Link
            to={`/post/${post.uuid}`}
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
              {withoutUrls}
            </p>
          ) : (
            paragraphs.map((paragraph, paragraphIndex) => (
              <p key={`post-${post.uuid}-paragraph-${paragraphIndex}`}>
                {paragraph}
              </p>
            ))
          )}

          {mediaUrls.length > 0 && (
            <MediaCarousel
              postUuid={post.uuid}
              mediaUrls={mediaUrls}
              maxImageHeight={"600px"}
            />
          )}
        </div>

        <Interactions
          postUuid={post.uuid}
          likedByUser={post.likedByMe}
          likeCount={post.likeCount}
          replyCount={post.replyCount}
          shareable
        />
      </div>
    </div>
  );
}
