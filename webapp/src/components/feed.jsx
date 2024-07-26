import PostCard from "./post-card";

export default function Feed({ posts }) {
  return (
    <div className="vstack gap-2">
      {posts.map((post) => (
        <div key={`post-card-${post.id}`}>
          <PostCard post={post} truncate linkToPost />
        </div>
      ))}
    </div>
  );
}
