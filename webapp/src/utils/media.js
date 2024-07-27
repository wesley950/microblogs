const URL_REGEX = /https?:\/\/[^\s]+/g;

export function parseBody(body) {
  let mediaUrls = body.match(URL_REGEX) || [];
  return {
    paragraphs: body.split("\n").map((paragraph) => paragraph.trim()),
    mediaUrls,
  };
}
