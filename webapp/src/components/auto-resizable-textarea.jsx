export default function AutoResizableTextarea({ ...props }) {
  return (
    <textarea
      {...props}
      style={{
        resize: "none",
        overflow: "hidden",
        ...props.style,
      }}
      onInput={(ev) => {
        // resize textarea
        ev.target.style.height = "auto";
        ev.target.style.height = `${ev.target.scrollHeight}px`;
      }}
    />
  );
}
