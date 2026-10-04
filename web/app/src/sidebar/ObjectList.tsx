import { CubeSmallIcon } from "../shell/icons";
import { selectObject } from "../state/actions";
import { useApp } from "../state/store";

export function ObjectList() {
  const scene = useApp((state) => state.scene);
  const selected = useApp((state) => state.selectedObject);
  if (!scene) return <div className="side-empty">No objects</div>;
  return (
    <div className="object-list" role="tree" data-testid="object-list">
      <div className="object-list-head"><span>Name</span><span>Parts</span></div>
      {scene.objects.map((object) => (
        <div key={object.id} role="treeitem" aria-selected={selected === object.id}
          className={selected === object.id ? "object-row selected" : "object-row"}
          onClick={() => selectObject(selected === object.id ? null : object.id)}>
          <CubeSmallIcon />
          <span className="object-name">{object.name || `Object ${object.id}`}</span>
          <span className="object-count">{object.volumes.length}</span>
        </div>
      ))}
    </div>
  );
}
