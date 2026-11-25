import * as THREE from "three";
import { type PlateMaterials, plateMaterials } from "./bed";
import { ModelMaterials } from "./models";
import { optionMarkerMaterial } from "./optionMarkers";
import { toolpathMaterial } from "./toolpathMesh";
import type { ViewportColors } from "./Viewport";

/**
 * The application's one WebGL context and the materials whose programs live in it. Every canvas
 * draws with the context OpenGLManager::init_glcontext creates on its first call
 * (OpenGLManager.cpp:352-378); GLShadersManager::init (GLShadersManager.cpp:22) and
 * ViewerImpl::init (ViewerImpl.cpp:790-867) compile their programs once. A `Viewport` attaches the
 * renderer's canvas to its container. Three.js releases a program with the last material that
 * uses it, so these materials are never disposed.
 */
export interface SharedGl {
  readonly renderer: THREE.WebGLRenderer;
  readonly plateMaterials: PlateMaterials;
  readonly modelMaterials: ModelMaterials;
  readonly toolpaths: THREE.ShaderMaterial;
  readonly optionMarkers: THREE.ShaderMaterial;
  /** The toolpath and option-marker programs were sent to compile. */
  toolpathsCompiled: boolean;
}

let shared: SharedGl | null = null;

/** The shared context, created with the first viewport's theme colours. */
export function sharedGl(colors: ViewportColors): SharedGl {
  if (shared) return shared;
  // Cap the backing buffer at 2× and avoid a multisample resolve at that pixel density.
  const pixelRatio = Math.min(window.devicePixelRatio, 2);
  const renderer = new THREE.WebGLRenderer({ antialias: pixelRatio < 2, preserveDrawingBuffer: true });
  renderer.setPixelRatio(pixelRatio);
  shared = {
    renderer,
    plateMaterials: plateMaterials(colors),
    modelMaterials: new ModelMaterials(),
    toolpaths: toolpathMaterial(),
    optionMarkers: optionMarkerMaterial(),
    toolpathsCompiled: false,
  };
  return shared;
}
