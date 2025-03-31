import { ComponentProps, splitProps } from "solid-js";

import styles from "./Switchbox.module.sass";

export function Switchbox(props: ComponentProps<"input">) {
  const [switchProps, restProps] = splitProps(props, ["class", "classList"]);

  return (
    <input
      {...restProps}
      classList={{
        [styles.base]: true,
        [switchProps.class]: true,
        ...(switchProps.classList || {}),
      }}
      type="checkbox"
    />
  );
}
