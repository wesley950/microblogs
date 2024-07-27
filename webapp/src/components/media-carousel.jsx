import { Carousel } from "bootstrap";

export default function MediaCarousel({ postId, mediaUrls, maxImageHeight }) {
  return (
    <div
      className="carousel slide mb-2"
      id={`post-card-images-carousel-${postId}`}
    >
      <div className="carousel-inner">
        {mediaUrls.map((imageUrl, index) => (
          <div
            className={`carousel-item ${index === 0 ? "active" : ""}`}
            key={`post-${index}-image-${index}`}
            style={{
              height: `${maxImageHeight}`,
              minHeight: `${maxImageHeight}`,
              maxHeight: `${maxImageHeight}`,
            }}
          >
            <img
              src={imageUrl}
              className="object-fit-contain w-100 h-100 rounded bg-dark"
            />
          </div>
        ))}
      </div>
      {mediaUrls.length > 1 && (
        <>
          <button
            className="carousel-control-prev"
            type="button"
            data-bs-slide="prev"
            data-bs-target={`#post-card-images-carousel-${postId}`}
          >
            <span className="carousel-control-prev-icon" />
            <span className="visually-hidden">Anterior</span>
          </button>
          <button
            className="carousel-control-next"
            type="button"
            data-bs-slide="next"
            data-bs-target={`#post-card-images-carousel-${postId}`}
          >
            <span className="carousel-control-next-icon" />
            <span className="visually-hidden">Próximo</span>
          </button>
        </>
      )}
    </div>
  );
}
