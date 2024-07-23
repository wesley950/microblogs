export default function Interactions({ likeCount, replyCount }) {
  return (
    <div className="hstack gap-2 justify-content-evenly text-center">
      <button className="btn link-danger btn-md">
        <i className="bi bi-heart"></i> {likeCount} curtidas
      </button>
      <button className="btn link-primary btn-md">
        <i className="bi bi-chat"></i> {replyCount} respostas
      </button>
      <button className="btn link-info btn-md">
        <i className="bi bi-share"></i> Compartilhar
      </button>
    </div>
  );
}
