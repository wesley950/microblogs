import { forwardRef } from "react";

const AutoResizableTextarea = forwardRef((props, ref) => (
  <textarea
    {...props}
    ref={ref}
    style={{
      resize: "none",
      overflow: "hidden",
      ...props.style,
    }}
    onInput={(ev) => {
      // resize textarea
      ev.target.style.height = "auto";
      ev.target.style.height = `${ev.target.scrollHeight}px`;

      props.onInput?.(ev);
    }}
  />
));

export default AutoResizableTextarea;
