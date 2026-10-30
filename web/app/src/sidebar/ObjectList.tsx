import type { Scene, SceneObject, ScenePlate, VolumeType } from "@ares-slicer/wasm";
import { Fragment, type ReactNode, useState } from "react";
import { CubeSmallIcon, OrcaIcon } from "../shell/icons";
import { selectObject } from "../state/actions";
import { addPlate, deletePlate, MAX_PLATES, selectPlate } from "../state/plates";
import { useApp } from "../state/store";

/** MenuFactory::get_volume_bitmaps, indexed by ModelVolumeType. */
const VOLUME_ICON: Record<VolumeType, string> = {
  ModelPart: "menu_add_part",
  NegativeVolume: "menu_add_negative",
  ParameterModifier: "menu_add_modifier",
  SupportBlocker: "menu_support_blocker",
  SupportEnforcer: "menu_support_enforcer",
};

/** ObjectDataViewModel object node; volumes become child nodes once there is more than one. */
function ObjectNode({ object, name, selected, nested = false }: { object: SceneObject; name: string; selected: boolean; nested?: boolean }) {
  const [expanded, setExpanded] = useState(false);
  const parts = object.volumes.length > 1 ? object.volumes : [];
  const printable = object.instances.some((instance) => instance.printable);
  return (
    <>
      <div role="treeitem" aria-selected={selected} aria-expanded={parts.length ? expanded : undefined}
        className={`object-row${nested ? " object-nested" : ""}${selected ? " selected" : ""}${printable ? "" : " unprintable"}`}
        onClick={() => selectObject(selected ? null : object.id)}>
        <button className={`object-expander${parts.length ? "" : " hidden"}`} aria-label="Toggle parts" tabIndex={-1}
          onClick={(event) => {
            event.stopPropagation();
            setExpanded(!expanded);
          }} />
        <CubeSmallIcon />
        <span className="object-name">{name}</span>
        {object.instances.length > 1 && <span className="object-count" title="Instances">×{object.instances.length}</span>}
      </div>
      {expanded && parts.map((volume) => (
        <div key={volume.id} role="treeitem" className="object-row object-part" title={volume.type}
          onClick={() => selectObject(object.id)}>
          <OrcaIcon name={VOLUME_ICON[volume.type]} />
          <span className="object-name">{volume.name || `Part ${volume.id}`}</span>
        </div>
      ))}
    </>
  );
}

/** An object sits under the plate holding its first instance (ObjectDataViewModel::AddPlate, `contain_instance_totally(obj_idx, 0)`). */
const onPlate = ({ instances: [first] }: SceneObject, plate: ScenePlate) =>
  plate.instances.some(([id, instance]) => id === first.id[0] && instance === first.id[1]);

/**
 * The plater's plates, each listing its objects (ObjectList::reload_all_plates), and the "Outside"
 * node of objects on no plate (ObjectDataViewModel::AddOutsidePlate), shown while it has any.
 */
function PlateNodes({ scene, object }: { scene: Scene; object: (object: SceneObject, nested: boolean) => ReactNode }) {
  const current = useApp((state) => state.currentPlate);
  const slicing = useApp((state) => state.slice.status === "slicing");
  const outside = scene.objects.filter((item) => !scene.plates.some((plate) => onPlate(item, plate)));
  return (
    <>
      {scene.plates.map((plate, index) => (
        <Fragment key={plate.id}>
          <div role="treeitem" aria-selected={index === current} className={`object-row plate-row${index === current ? " selected" : ""}`}
            onClick={() => void selectPlate(index)}>
            <span className="object-name">Plate {plate.id}</span>
            {/* PartPlate delete icon (plate_close), shown while the list has more than one plate. */}
            {scene.plates.length > 1 && (
              <button className="plate-delete" aria-label={`Delete plate ${plate.id}`} title="Remove this plate" disabled={slicing}
                onClick={(event) => {
                  event.stopPropagation();
                  void deletePlate(index);
                }}>×</button>
            )}
          </div>
          {scene.objects.filter((item) => onPlate(item, plate)).map((item) => object(item, true))}
        </Fragment>
      ))}
      {outside.length > 0 && (
        <>
          <div role="treeitem" aria-selected={false} className="object-row plate-row">
            <span className="object-name">Outside</span>
          </div>
          {outside.map((item) => object(item, true))}
        </>
      )}
    </>
  );
}

export function ObjectList() {
  const scene = useApp((state) => state.scene);
  const selected = useApp((state) => state.selectedObject);
  const fileName = useApp((state) => state.file?.name);
  if (!scene) return <div className="side-empty">No objects</div>;
  const node = (object: SceneObject, nested: boolean) => (
    <ObjectNode key={`${nested}-${object.id}`} object={object} selected={selected === object.id} nested={nested}
      // load_stl names the object after the file; the wasm STL scene does not carry it.
      name={(scene.format === "stl" ? fileName : object.name) || `Object ${object.id}`} />
  );
  return (
    <div className="object-list" role="tree" data-testid="object-list">
      {scene.plates.length ? <PlateNodes scene={scene} object={node} /> : scene.objects.map((object) => node(object, false))}
    </div>
  );
}

/** "Add plate" (GLToolbar `addplate`, Plater::priv::on_action_add_plate), up to `MAX_PLATES`. */
export function AddPlateButton() {
  const count = useApp((state) => (state.plater ? state.scene?.plates.length ?? 0 : null));
  const slicing = useApp((state) => state.slice.status === "slicing");
  if (count === null) return null;
  return (
    <button className="plate-add" onClick={() => void addPlate()} disabled={slicing || count >= MAX_PLATES} title="Add plate">
      Add plate
    </button>
  );
}
