import { createSignal } from "solid-js";
import { CodeMirror } from "@solid-codemirror/codemirror";
import { basicSetup } from "codemirror";
import { rust } from "@codemirror/lang-rust";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";

import styles from "./CodePanel.module.sass";

const rsgroundTheme = HighlightStyle.define([
  { tag: t.keyword, class: styles.keyword },
  {
    tag: [
      t.function(t.name),
      t.function(t.propertyName),
      t.labelName,
      t.macroName,
    ],
    class: styles.fn,
  },
  { tag: [t.typeName, t.namespace], class: styles.struct },
  { tag: t.string, class: styles.string },
  { tag: t.meta, class: styles.attribute },
  { tag: t.special(t.variableName), class: styles.lifetime },
]);

export function CodePanel() {
  const [code, setCode] = createSignal(
    'pub struct Something {\n  prop: String\n}\n\n#[tokio::main]\nfn main<\'a>() {\n  let a = String::new();\n  println!(r#"{a}"#);\n}',
  );

  return (
    <CodeMirror
      class={styles.container}
      value={code()}
      extensions={[basicSetup, syntaxHighlighting(rsgroundTheme), rust()]}
    />
  );
}
