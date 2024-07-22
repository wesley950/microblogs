export default function UserAvatar({ username, realName }) {
  const seed = encodeURI(`${username}${realName}`);
  return (
    <>
      <div className="hstack gap-2">
        <img
          src={`https://api.dicebear.com/9.x/open-peeps/svg?seed=${seed}`}
          className="rounded-circle"
          width={64}
        />
        <div className="vstack gap-2">
          <h5 className="card-title">{username}</h5>
          <h6 className="card-subtitletext-body-secondary">{realName}</h6>
        </div>
      </div>
    </>
  );
}
