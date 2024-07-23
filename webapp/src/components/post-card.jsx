import { Carousel } from "bootstrap";
import { Link } from "react-router-dom";
import UserAvatar from "./user-avatar";
import Interactions from "./interactions";
import ImageCarousel from "./image-carousel";

export default function PostCard({ post, truncate = false }) {
  return (
    <div className="card">
      <div className="card-body">
        <div className="position-relative vstack gap-2">
          <Link
            to={`/post/${post.id}`}
            className="text-decoration-none stretched-link"
          >
            <UserAvatar
              username={post.user.username}
              realName={post.user.realName}
            />
          </Link>
          <p
            className="text-wrap overflow-hidden"
            style={{
              maxHeight: truncate ? "100px" : "none",
            }}
          >
            {post.body}
          </p>

          <ImageCarousel
            postId={post.id}
            imageUrls={post.imageUrls}
            maxImageHeight={"600px"}
          />
        </div>

        <Interactions
          likeCount={post.likeCount}
          replyCount={post.replyCount}
          shareable
        />
      </div>
    </div>
  );
}
