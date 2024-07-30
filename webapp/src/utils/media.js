const URL_REGEX = /https?:\/\/[^\s]+/g;

export function parseBody(body) {
  let mediaUrls = body.match(URL_REGEX) || [];
  let withoutUrls = body.replace(URL_REGEX, "");
  return {
    withoutUrls,
    paragraphs: withoutUrls
      .split("\n")
      .filter((paragraph) => paragraph !== "")
      .map((paragraph) => paragraph.trim()),
    mediaUrls,
  };
}

export function parsePost(post) {
  return {
    uuid: post.uuid,
    body: post.body,
    createdAt: new Date(post.created_at),
    likeCount: post.like_count,
    replyCount: post.reply_count,
    likedByMe: post.liked_by_user,
    user: {
      username: post.poster.username,
      realName: post.poster.real_name,
    },
  };
}
