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
