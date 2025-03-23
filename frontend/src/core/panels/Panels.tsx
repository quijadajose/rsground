import "dockview-core/dist/styles/dockview.css";

import {
  DockviewComponent,
  DockviewTheme,
  IContentRenderer,
  ITabRenderer,
} from "dockview-core";

import styles from "./Panels.module.sass";
import "./dockview.sass";

export function Panels() {
  const element = <div class={styles.container} /> as HTMLElement;

  const dockview = new DockviewComponent(element, {
    theme: {
      name: "rsground",
      className: "rsground-dockview",
      gap: 10,
      dndOverlayMounting: "absolute",
      dndPanelOverlay: "group",
    } satisfies DockviewTheme,
    disableFloatingGroups: true,
    singleTabMode: "default",

    createComponent(options) {
      const element = (
        <div class={styles.panel}>CHILD {options.id}-{options.name}</div>
      ) as HTMLElement;

      return {
        element,
        init(_params) {},
      } satisfies IContentRenderer;
    },

    createTabComponent(options) {
      const element = (
        <div class={styles.tab}>TAB {options.id}-{options.name}</div>
      ) as HTMLElement;

      return {
        element,
        init(_params) {
        },
      } satisfies ITabRenderer;
    },
  });

  dockview.api.addPanel({
    id: "welcome-1",
    component: "welcome",
    title: "Welcome",
  });
  dockview.api.addPanel({
    id: "welcome-2",
    component: "welcome",
    title: "Welcome",
    position: { referencePanel: "welcome-1", direction: "right" },
  });

  return element;
}
