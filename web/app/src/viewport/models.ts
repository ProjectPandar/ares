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

/** One group per object instance, holding a mesh per volume. */
export function buildInstances(scene: Scene, filamentColour: string): THREE.Group[] {
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
        const [color, opacity] = volume.type === "ModelPart"
          ? [new THREE.Color(filamentColour).getHex(), 1]
          : VOLUME_STYLE[volume.type];
        const material = new THREE.MeshStandardMaterial({
          color,
          opacity,
          transparent: opacity < 1,
          depthWrite: opacity === 1,
          flatShading: true,
          roughness: 0.75,
          metalness: 0.05,
          side: THREE.DoubleSide,
        });
        material.userData.baseColor = color;
        const mesh = new THREE.Mesh(geometry, material);
        mesh.matrixAutoUpdate = false;
        mesh.matrix.fromArray(volume.matrix);
        mesh.userData.objectId = object.id;
        group.add(mesh);
      }
      groups.push(group);
    }
  }
  return groups;
}
