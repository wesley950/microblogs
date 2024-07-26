import { Link } from "react-router-dom";
import UserAvatar from "./user-avatar";
import Interactions from "./interactions";
import ImageCarousel from "./image-carousel";

export default function PostCard({
  post,
  truncate = false,
  linkToPost = false,
}) {
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
            post.body
              .split("\n")
              .map((paragraph, index) => (
                <p key={`post-${post.id}-paragraph-${index}`}>{paragraph}</p>
              ))
          )}

          {post.imageUrls.length > 0 && (
            <ImageCarousel
              postId={post.id}
              imageUrls={post.imageUrls}
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
