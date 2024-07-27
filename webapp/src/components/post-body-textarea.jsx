import { useEffect, useRef, useState } from "react";
import { parseBody } from "../utils/media";
import AutoResizableTextarea from "./auto-resizable-textarea";

export default function PostBodyTextarea({ placeholder, name = "body" }) {
  const [previewMediaUrls, setPreviewMediaUrls] = useState([]);
  const ref = useRef(null);

  useEffect(() => {
    if (ref.current && ref.current?.value === "") {
      setPreviewMediaUrls([]);
    }
  }, [ref.current, ref.current?.value]);

  return (
    <>
      <div className="form-floating">
        <AutoResizableTextarea
          className="form-control"
          name={name}
          ref={ref}
          placeholder=""
          onInput={(e) => {
            const text = e.target.value || "";
            const { mediaUrls } = parseBody(text);
            setPreviewMediaUrls(mediaUrls);
          }}
        />
        <label>{placeholder}</label>
      </div>
      <div className="hstack gap-1 d-flex flex-row flex-wrap">
        {previewMediaUrls.map((mediaUrl, index) => (
          <img
            src={mediaUrl}
            key={`post-form-media-preview-${index}`}
            width={80}
            className="img-thumbnail img-fluid"
          />
        ))}
      </div>
    </>
  );
}
