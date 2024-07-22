export default function Interactions({ likes, comments, shareable }) {
  return (
    <div className="hstack gap-2 justify-content-evenly text-center">
      <button className="btn link-danger btn-md">
        <i className="bi bi-heart"></i> {likes} curtidas
      </button>
      {comments && (
        <button className="btn link-primary btn-md">
          <i className="bi bi-chat"></i> {comments} comentários
        </button>
      )}
      {shareable && (
        <button className="btn link-info btn-md">
          <i className="bi bi-share"></i> Compartilhar
        </button>
      )}
    </div>
  );
}
