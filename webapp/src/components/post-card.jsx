import { Carousel } from "bootstrap";
import { Link } from "react-router-dom";
import UserAvatar from "./user-avatar";
import Interactions from "./interactions";
import ImageCarousel from "./image-carousel";

export default function PostCard({ post }) {
  return (
    <div className="card">
      <div className="card-body">
        <div className="position-relative vstack gap-2">
          <Link
            to={`/post/${post.id}`}
            className="text-decoration-none stretched-link"
          >
            <UserAvatar
              username={post.poster.username}
              realName={post.poster.realName}
            />
          </Link>
          <p
            className="text-wrap overflow-hidden"
            style={{
              maxHeight: "100px",
            }}
          >
            {post.body}
          </p>

          <ImageCarousel postId={post.id} imageUrls={post.imageUrls} maxImageHeight={"600px"} />
        </div>

        <Interactions likes={post.likes} comments={post.commentCount} shareable />
      </div>
    </div>
  );
}
