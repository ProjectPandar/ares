import type { SceneObject, VolumeType } from "@ares-slicer/wasm";
import { useState } from "react";
import { CubeSmallIcon, OrcaIcon } from "../shell/icons";
import { selectObject } from "../state/actions";
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
function ObjectNode({ object, name, selected }: { object: SceneObject; name: string; selected: boolean }) {
  const [expanded, setExpanded] = useState(false);
  const parts = object.volumes.length > 1 ? object.volumes : [];
  const printable = object.instances.some((instance) => instance.printable);
  return (
    <>
      <div role="treeitem" aria-selected={selected} aria-expanded={parts.length ? expanded : undefined}
        className={`object-row${selected ? " selected" : ""}${printable ? "" : " unprintable"}`}
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

export function ObjectList() {
  const scene = useApp((state) => state.scene);
  const selected = useApp((state) => state.selectedObject);
  const fileName = useApp((state) => state.file?.name);
  if (!scene) return <div className="side-empty">No objects</div>;
  return (
    <div className="object-list" role="tree" data-testid="object-list">
      {scene.objects.map((object) => (
        <ObjectNode key={object.id} object={object} selected={selected === object.id}
          // load_stl names the object after the file; the wasm STL scene does not carry it.
          name={(scene.format === "stl" ? fileName : object.name) || `Object ${object.id}`} />
      ))}
    </div>
  );
}
