export default function UserAvatar({ username, realName }) {
  const seed = encodeURI(`${username}${realName}`);
  return (
    <>
      <div className="hstack gap-2">
        <img
          src={`https://api.dicebear.com/9.x/open-peeps/svg?seed=${seed}&backgroundColor=b6e3f4,c0aede,d1d4f9,ffd5dc,ffdfbf`}
          className="rounded-circle img-thumbnail img-fluid"
          width={64}
        />
        <div className="vstack my-auto">
          <h5 className="card-title my-0">{username}</h5>
          <h6 className="card-subtitletext-body-secondary my-0">{realName}</h6>
        </div>
      </div>
    </>
  );
}
