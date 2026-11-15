import type { Scene, VolumeType } from "@ares-slicer/wasm";
import * as THREE from "three";
import type { InstanceRef } from "./instanceDrag";

// GLVolume::render colours for non-part volumes (3DScene.cpp).
const VOLUME_STYLE: Record<Exclude<VolumeType, "ModelPart">, [number, number]> = {
  NegativeVolume: [0x606060, 0.45],
  ParameterModifier: [0xffff00, 0.25],
  SupportEnforcer: [0x4d80ff, 0.45],
  SupportBlocker: [0xff4d4d, 0.45],
};

const SELECTED = new THREE.Color(0xffa040);

/** A volume colour, blended towards the selection colour when its object is selected. */
const shade = (color: number, selected: boolean) => (selected ? new THREE.Color(color).lerp(SELECTED, 0.65) : new THREE.Color(color));

/**
 * The instance materials, one per volume type and selection state, built on first use and kept for the
 * viewport's life: the objects draw with the shader GLCanvas3D::_render_objects looks up by name
 * (GLCanvas3D.cpp:9677-9679), which GLShadersManager::init compiles once (GLShadersManager.cpp:22).
 */
export class ModelMaterials {
  readonly #materials = new Map<string, THREE.MeshStandardMaterial>();
  #filament = 0;

  get(type: VolumeType, selected: boolean) {
    const key = `${type}:${selected}`;
    const kept = this.#materials.get(key);
    if (kept) return kept;
    const [color, opacity] = type === "ModelPart" ? [this.#filament, 1] : VOLUME_STYLE[type];
    const material = new THREE.MeshStandardMaterial({
      opacity,
      transparent: opacity < 1,
      depthWrite: opacity === 1,
      flatShading: true,
      roughness: 0.75,
      metalness: 0.05,
      side: THREE.DoubleSide,
    });
    material.color.copy(shade(color, selected));
    this.#materials.set(key, material);
    return material;
  }

  /** The model-part colour, from the first filament. */
  setFilament(colour: string) {
    this.#filament = new THREE.Color(colour).getHex();
    for (const selected of [false, true]) this.#materials.get(`ModelPart:${selected}`)?.color.copy(shade(this.#filament, selected));
  }

  dispose() {
    for (const material of this.#materials.values()) material.dispose();
  }
}

/** One group per object instance, holding a mesh per volume. */
export function buildInstances(scene: Scene, materials: ModelMaterials): THREE.Group[] {
  const groups: THREE.Group[] = [];
  for (const [objectIndex, object] of scene.objects.entries()) {
    const geometries = object.volumes.map((volume) => {
      const geometry = new THREE.BufferGeometry();
      const start = volume.vertexOffset * 3;
      geometry.setAttribute(
        "position",
        new THREE.BufferAttribute(scene.positions.subarray(start, start + volume.vertexCount * 3), 3),
      );
      geometry.setIndex(new THREE.BufferAttribute(
        scene.indices.subarray(volume.indexOffset, volume.indexOffset + volume.indexCount), 1,
      ));
      return { volume, geometry };
    });
    for (const [instanceIndex, instance] of object.instances.entries()) {
      const group = new THREE.Group();
      group.matrixAutoUpdate = false;
      group.matrix.fromArray(instance.matrix);
      group.userData.objectId = object.id;
      group.userData.ref = { object: objectIndex, instance: instanceIndex } satisfies InstanceRef;
      for (const { volume, geometry } of geometries) {
        const mesh = new THREE.Mesh(geometry, materials.get(volume.type, false));
        mesh.matrixAutoUpdate = false;
        mesh.matrix.fromArray(volume.matrix);
        mesh.userData.objectId = object.id;
        mesh.userData.volumeType = volume.type;
        group.add(mesh);
      }
      groups.push(group);
    }
  }
  return groups;
}
