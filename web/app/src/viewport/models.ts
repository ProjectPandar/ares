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
const shade = (color: number, selected: boolean) =>
  selected ? new THREE.Color(color).lerp(SELECTED, 0.65) : new THREE.Color(color);

/**
 * The instance materials, one per volume type and selection state, built on first use and kept for the
 * context's life (`sharedGl`): the objects draw with the shader GLCanvas3D::_render_objects looks up by name
 * (GLCanvas3D.cpp:9677-9679), which GLShadersManager::init compiles once (GLShadersManager.cpp:22).
 */
export class ModelMaterials {
  readonly #materials = new Map<string, THREE.MeshStandardMaterial>();
  #filaments: number[] = [];

  get(type: VolumeType, selected: boolean, slot: number) {
    const color = this.#filaments[slot - 1] ?? this.#filaments[0] ?? 0x29b2a3;
    const [tint, opacity] = type === "ModelPart" ? [color, 1] : VOLUME_STYLE[type];
    const key = `${type}:${selected}:${type === "ModelPart" ? slot : 0}`;
    const kept = this.#materials.get(key);
    if (kept) {
      kept.color.copy(shade(tint, selected));
      return kept;
    }
    const material = new THREE.MeshStandardMaterial({
      opacity,
      transparent: opacity < 1,
      depthWrite: opacity === 1,
      flatShading: true,
      roughness: 0.75,
      metalness: 0.05,
      side: THREE.DoubleSide,
    });
    material.color.copy(shade(tint, selected));
    this.#materials.set(key, material);
    return material;
  }

  setFilaments(colours: readonly string[]) {
    this.#filaments = colours.map((colour) => new THREE.Color(colour || "#29b2a3").getHex());
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
      geometry.setIndex(
        new THREE.BufferAttribute(
          scene.indices.subarray(volume.indexOffset, volume.indexOffset + volume.indexCount),
          1,
        ),
      );
      return { volume, geometry };
    });
    for (const [instanceIndex, instance] of object.instances.entries()) {
      const group = new THREE.Group();
      group.matrixAutoUpdate = false;
      group.matrix.fromArray(instance.matrix);
      group.userData.objectId = object.id;
      group.userData.ref = { object: objectIndex, instance: instanceIndex } satisfies InstanceRef;
      for (const { volume, geometry } of geometries) {
        const mesh = new THREE.Mesh(geometry, materials.get(volume.type, false, volume.effectiveFilament));
        mesh.matrixAutoUpdate = false;
        mesh.matrix.fromArray(volume.matrix);
        mesh.userData.objectId = object.id;
        mesh.userData.volumeType = volume.type;
        mesh.userData.volumeId = volume.id;
        mesh.userData.filament = volume.effectiveFilament;
        group.add(mesh);
      }
      groups.push(group);
    }
  }
  return groups;
}

/** Plater edits: updates instance matrices and effective filaments when mesh buffers are unchanged. */
export function updateInstances(models: THREE.Group, scene: Scene): boolean {
  const groups = models.children;
  const matrices = scene.objects.flatMap((object) => object.instances.map((instance) => instance.matrix));
  if (groups.length !== matrices.length) return false;
  groups.forEach((group, index) => group.matrix.fromArray(matrices[index]));
  models.updateMatrixWorld(true);
  const objects = new Map(scene.objects.map((object) => [object.id, object]));
  models.traverse((node) => {
    if (!(node instanceof THREE.Mesh)) return;
    const object = objects.get(node.userData.objectId as number);
    const volume = object?.volumes.find(({ id }) => id === node.userData.volumeId);
    if (volume) node.userData.filament = volume.effectiveFilament;
  });
  return true;
}

/** Assigns each model mesh its material for the current selection and filament palette. */
export function applySelectionMaterials(models: THREE.Group, materials: ModelMaterials, objectId: number | null) {
  models.traverse((node) => {
    if (node instanceof THREE.Mesh)
      node.material = materials.get(
        node.userData.volumeType as VolumeType,
        node.userData.objectId === objectId,
        node.userData.filament as number,
      );
  });
}
