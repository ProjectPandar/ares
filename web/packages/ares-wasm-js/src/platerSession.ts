import type { PresetCatalog } from "./presetCatalog.js";
import type { PlaterEdit, RequestBody } from "./protocol.js";
import type { SliceResult } from "./toolpaths.js";
import type { ArrangeSettings, ArrangeSettingsModel, PlaterLayout, ProfileSelection, ProfileSliceOptions, ScaleAndSize, ScaleEdit, Scene } from "./types.js";

type Call = (body: RequestBody) => Promise<unknown>;
type Vec3 = [number, number, number];
type Axis = 0 | 1 | 2;

/**
 * An editable plater held in the slicer worker, like OrcaSlicer's `Plater` with its
 * `Model` and `PartPlateList`: instance moves and rotations, plate add/delete, and per-plate
 * slicing of the edited project. `object` and `instance` index `Scene.objects` and their
 * `instances`; coordinates are world (plater scene) millimetres. Each edit resolves to the
 * new layout. Create with `AresSlicer.createPlaterSession`; `free` it when replaced.
 */
export class PlaterSession {
  readonly #call: Call;
  readonly #sessionId: number;

  /** @internal */
  constructor(call: Call, sessionId: number) {
    this.#call = call;
    this.#sessionId = sessionId;
  }

  /** The edited project as `loadScene` returns it, with every plate's `origin`. */
  async scene(): Promise<Scene> {
    return (await this.#call({ kind: "platerScene", sessionId: this.#sessionId })) as Scene;
  }

  /**
   * Moves an instance by `displacement` (`Selection::translate`), then drops it onto the bed
   * unless it now sinks into it (`GLCanvas3D::do_move`) and reassigns it to the plate it lies on.
   */
  translate(object: number, instance: number, displacement: Vec3): Promise<PlaterLayout> {
    return this.#edit({ op: "translate", object, instance, displacement });
  }

  /** Sets one world position coordinate of an instance (the Position fields, `change_position_value`). */
  setPosition(object: number, instance: number, axis: Axis, value: number): Promise<PlaterLayout> {
    return this.#edit({ op: "setPosition", object, instance, axis, value });
  }

  /**
   * Rotates an instance by `radians` about the world axes through its bounding-sphere centre
   * (`Selection::rotate`), then drops it onto the bed unless it sank before and after
   * (`GLCanvas3D::do_rotate`).
   */
  rotate(object: number, instance: number, radians: Vec3): Promise<PlaterLayout> {
    return this.#edit({ op: "rotate", object, instance, radians });
  }

  /**
   * Hands the session the installed system vendors (`PresetBundle::vendors`): the catalog built from
   * the installed vendor profiles. The arrange reads the printer's vendor from it, so a Bambu Lab
   * printer counts as one only while BBL is installed. Call it after creating the session and
   * after every change of the installed vendors; until then no vendor is installed.
   */
  async setInstalledVendors(catalog: PresetCatalog): Promise<void> {
    await this.#call({ kind: "platerSetInstalledVendors", sessionId: this.#sessionId, catalogId: catalog.workerId });
  }

  /** The instance's absolute rotation fields in degrees (`GizmoObjectManipulation`). */
  async absoluteRotation(object: number, instance: number): Promise<Vec3> {
    return (await this.#call({ kind: "platerAbsoluteRotation", sessionId: this.#sessionId, object, instance })) as Vec3;
  }

  /** Sets one absolute rotation field, rotating by the change (`change_absolute_rotation_value`). */
  setAbsoluteRotation(object: number, instance: number, axis: Axis, degrees: number): Promise<PlaterLayout> {
    return this.#edit({ op: "setAbsoluteRotation", object, instance, axis, degrees });
  }

  /**
   * "Scale to build volume" (`Selection::scale_to_fit_print_volume`): the instance scales uniformly to
   * fill the current plate's free build volume and is centred there; its object's other instances
   * take the same scale. Refused on a circular bed, where it is not ported.
   */
  scaleToFit(object: number, instance: number): Promise<PlaterLayout> {
    return this.#edit({ op: "scaleToFit", object, instance });
  }

  /** Whether "Scale to build volume" is ported for the session's bed: every bed but a circular one. */
  async scaleToFitPorted(): Promise<boolean> {
    return (await this.#call({ kind: "platerScaleToFitPorted", sessionId: this.#sessionId })) as boolean;
  }

  /** The instance's Scale (percent) and Size (mm) fields in world coordinates (`GizmoObjectManipulation`). */
  async scaleAndSize(object: number, instance: number): Promise<ScaleAndSize> {
    return (await this.#call({ kind: "platerScaleAndSize", sessionId: this.#sessionId, object, instance })) as ScaleAndSize;
  }

  /**
   * Sets one Scale field in percent (`change_scale_value`): the instance scales about its box centre,
   * its object's other instances take the same scale, and each drops onto the bed unless it stays
   * sinking (`GLCanvas3D::do_scale`). A value at or below zero, or the shown one, changes nothing.
   */
  setScale(object: number, instance: number, edit: ScaleEdit): Promise<PlaterLayout> {
    return this.#edit({ op: "setScale", object, instance, edit });
  }

  /** Sets one Size field in millimetres (`change_size_value`), scaling as `setScale` does. */
  setSize(object: number, instance: number, edit: ScaleEdit): Promise<PlaterLayout> {
    return this.#edit({ op: "setSize", object, instance, edit });
  }

  /**
   * Mirrors an instance along `axis` about its centre (the object menu's "Mirror",
   * `Selection::mirror`); its object's other instances follow, and each drops onto the bed unless
   * it stays sinking (`GLCanvas3D::do_mirror`).
   */
  mirror(object: number, instance: number, axis: Axis): Promise<PlaterLayout> {
    return this.#edit({ op: "mirror", object, instance, axis });
  }

  /**
   * "Add instance" (`Plater::increase_instances`): `num` copies of object `object`'s last instance,
   * each moved by 5% of the larger bed side from the previous. Rejects when an instance is not
   * printable.
   */
  increaseInstances(object: number, num = 1): Promise<PlaterLayout> {
    return this.#edit({ op: "increaseInstances", object, num });
  }

  /** "Remove instance" (`Plater::decrease_instances`): object `object`'s last `num` instances. */
  decreaseInstances(object: number, num = 1): Promise<PlaterLayout> {
    return this.#edit({ op: "decreaseInstances", object, num });
  }

  /**
   * The object menu's "Delete" (`Plater::priv::delete_object_from_model`): object `object` leaves the
   * model and its plates; later objects move up one index.
   */
  deleteObject(object: number): Promise<PlaterLayout> {
    return this.#edit({ op: "deleteObject", object });
  }

  /** "Delete" of one instance (`Plater::remove_selected`); a lone instance deletes its object. */
  deleteInstance(object: number, instance: number): Promise<PlaterLayout> {
    return this.#edit({ op: "deleteInstance", object, instance });
  }

  /** "Delete All" (`Plater::priv::delete_all_objects_from_model`): every object leaves; the plates stay. */
  deleteAllObjects(): Promise<PlaterLayout> {
    return this.#edit({ op: "deleteAllObjects" });
  }

  /** The plate menu's "Delete All" (`Selection::remove_curr_plate`) on the current plate. */
  deletePlateObjects(): Promise<PlaterLayout> {
    return this.#edit({ op: "deletePlateObjects" });
  }

  /**
   * "Set number of instances" (`Plater::set_number_of_copies`): instances added or removed to reach
   * `num` (0 to 1000); 0 removes the object unless it has a single instance.
   */
  setNumberOfCopies(object: number, num: number): Promise<PlaterLayout> {
    return this.#edit({ op: "setNumberOfCopies", object, num });
  }

  /** Appends a plate to the grid (`PartPlateList::create_plate`). */
  addPlate(): Promise<PlaterLayout> {
    return this.#edit({ op: "addPlate" });
  }

  /**
   * Deletes the plate at 0-based `index` with its instances' assignment; later plates and their
   * instances move up one place (`PartPlateList::delete_plate`). The sole plate stays.
   */
  deletePlate(index: number): Promise<PlaterLayout> {
    return this.#edit({ op: "deletePlate", index });
  }

  /**
   * Makes the plate at 0-based `index` the current plate (`PartPlateList::select_plate`), whose print
   * "Arrange all" reads its per-object skirt offset from.
   */
  selectPlate(index: number): Promise<PlaterLayout> {
    return this.#edit({ op: "selectPlate", index });
  }

  /**
   * Locks or unlocks the plate at 0-based `index` (`PartPlateList::lock_plate`). "Arrange all" leaves
   * a locked plate's instances where they are, and the plate's own arrange rejects.
   */
  lockPlate(index: number, locked: boolean): Promise<PlaterLayout> {
    return this.#edit({ op: "lockPlate", index, locked });
  }

  /**
   * Arranges every plate, or with `plate` (0-based) that plate alone (`ArrangeJob`). Rejects with the
   * message when the project needs what is not ported; the plater then stays as it was.
   */
  arrange(settings: ArrangeSettings, plate?: number): Promise<PlaterLayout> {
    return this.#edit({ op: "arrange", plate, settings });
  }

  /** The arrange settings defaults, their app-config key and the options the popup offers. */
  async arrangeSettings(): Promise<ArrangeSettingsModel> {
    return (await this.#call({ kind: "platerArrangeSettings", sessionId: this.#sessionId })) as ArrangeSettingsModel;
  }

  /** `settings` as the arrange popup leaves them (`GLCanvas3D::_render_arrange_menu`). */
  async shownArrangeSettings(settings: ArrangeSettings): Promise<ArrangeSettings> {
    return (await this.#call({ kind: "platerShownArrangeSettings", sessionId: this.#sessionId, settings })) as ArrangeSettings;
  }

  /** Slices plate `plate` (1-based, the first when omitted) as `AresSlicer.sliceProject` slices a project. */
  async slice(plate?: number, overrides: Record<string, string | string[]> = {}): Promise<SliceResult> {
    return (await this.#call({ kind: "platerSlice", sessionId: this.#sessionId, plate, overrides: JSON.stringify(overrides) })) as SliceResult;
  }

  /** Slices plate `options.plate` with selected presets, as `AresSlicer.sliceWithProfiles` slices the loaded file. */
  async sliceWithProfiles(catalogJson: string, selection: ProfileSelection, options: ProfileSliceOptions): Promise<SliceResult> {
    return (await this.#call({
      kind: "platerSliceWithProfiles",
      sessionId: this.#sessionId,
      plate: options.plate,
      catalog: catalogJson,
      selection: JSON.stringify(selection),
      options: JSON.stringify(options),
    })) as SliceResult;
  }

  /**
   * Makes the latest slice of plate `plate` (1-based) the toolpaths `AresSlicer.viewRange` and the
   * other view calls apply to, as switching plates reloads that plate's G-code preview
   * (`PartPlateList::get_current_slice_result`). Each plate's latest slice is kept until an edit.
   * Resolves `false` when the plate has no slice to show.
   */
  async preview(plate: number): Promise<boolean> {
    return (await this.#call({ kind: "platerPreview", sessionId: this.#sessionId, plate })) as boolean;
  }

  /** Releases the worker's session. */
  async free(): Promise<void> {
    await this.#call({ kind: "platerFree", sessionId: this.#sessionId });
  }

  async #edit(edit: PlaterEdit): Promise<PlaterLayout> {
    return (await this.#call({ kind: "platerEdit", sessionId: this.#sessionId, edit })) as PlaterLayout;
  }
}
