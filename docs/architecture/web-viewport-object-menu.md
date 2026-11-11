# Web viewport object menu

## Source boundary

This slice ports OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c` GUI behaviour. Paths are
relative to `src/slic3r/GUI/`.

- `GLCanvas3D::on_mouse` on a right button release (`GLCanvas3D.cpp:5411-5471`):
  - over an object, the hovered volume's instance is added to the selection and the canvas posts
    `EVT_GLCANVAS_RIGHT_CLICK` at the cursor;
  - over a plate or empty space, the selection is cleared (`:5437-5451`), and a plate shows the
    plate menu;
  - a release after a camera rotation or pan posts nothing (`m_mouse.ignore_right_up`, set at
    `:5313, 5348`).
- `Plater::priv::on_right_click` (`Plater.cpp:14201-14263`) shows `MenuFactory::object_menu`
  (`GUI_Factories.cpp:2122`) for a selected full instance or object, at the cursor
  (`show_right_click_menu`, `:14183`). It is the object list's menu (`GUI_ObjectList.cpp`), so the
  canvas shows the web's `sidebar/ObjectMenu`.

## Web destination

- `web/app/src/viewport/Viewport.ts`: a right button release within the left click's 4 px of its
  press reports the picked object and the cursor (`onContextPick`); the canvas prevents the
  browser's context menu.
- `web/app/src/viewport/ViewportView.tsx`: on the Prepare tab, the picked object is selected and
  `ObjectMenu` opens at the cursor, with its "Set number of instances" dialog. A click elsewhere
  clears the selection and shows no menu. The Preview tab has no object menu.

## Deferred

- The default menu of an empty selection and the plate menu (`menus.default_menu`,
  `on_plate_right_click`); the canvas shows no menu there.
- The multiple selection and part menus, and the instance selection of a right click: the web
  canvas selects whole objects, so the menu acts on the object as the object list's does.
- Orca shows no menu while a gizmo is open (`m_gizmos.get_current_type()`); the web canvas always
  has the move or rotate gizmo, so the menu opens with either.

## Verification

`web/e2e/viewportObjectMenu.spec.ts` right-clicks the cube of the two-plate project. The object
must be selected and the menu must open at the cursor. "Add instance" and "Set number of
instances" must change the object, and Escape must close the menu. A right drag must pan the
camera without a menu. A right click on empty space must clear the selection, and the Preview tab
must show no menu. Every canvas `contextmenu` event must be prevented.
