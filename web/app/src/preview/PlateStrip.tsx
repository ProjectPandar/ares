import { selectPlate } from "../state/plates";
import { useApp } from "../state/store";

/**
 * The Preview's plate toolbar (`GLCanvas3D::_render_imgui_select_plate_toolbar`): each plate's
 * thumbnail with its number, marked while it has no slice; a click selects the plate.
 */
export function PlateStrip() {
  const plates = useApp((state) => state.scene!.plates);
  const thumbnails = useApp((state) => state.plateThumbnails);
  const currentPlate = useApp((state) => state.currentPlate);
  const plateSlices = useApp((state) => state.plateSlices);
  return (
    <nav className="plate-strip" aria-label="Plates">
      {plates.map((plate, index) => {
        const sliced = index in plateSlices;
        return (
          <button
            key={plate.id}
            className="plate-thumb"
            aria-current={index === currentPlate}
            aria-label={`Plate ${index + 1}`}
            title={sliced ? `Plate ${index + 1}` : `Plate ${index + 1} · not sliced`}
            onClick={() => void selectPlate(index)}
          >
            {thumbnails[index] && <img src={thumbnails[index]} alt="" draggable={false} />}
            <span className="plate-thumb-index">{index + 1}</span>
            {!sliced && <span className="plate-thumb-status">Not sliced</span>}
          </button>
        );
      })}
    </nav>
  );
}
