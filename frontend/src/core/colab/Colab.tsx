import Dialog from "@corvu/dialog";
import Popover from "@corvu/popover";
import {
  FaBrandsGithub,
  FaSolidChevronDown,
  FaSolidLock,
  FaSolidUser,
} from "solid-icons/fa";

import { isColabOpen, setIsColabOpen } from "./store";

import styles from "./Colab.module.sass";
import { For } from "solid-js";

export function Colab() {
  const requestUsers = ["CHIWO", "Jopzgo", "gg0074x", "Otro"];

  return (
    <Dialog open={isColabOpen()} onOpenChange={setIsColabOpen}>
      <Dialog.Portal>
        <Dialog.Overlay class={styles.overlay} />
        <Dialog.Content class={styles.content}>
          <h2 class={styles.title}>Live Collaboration</h2>

          <div class={styles.container}>
            <div>
              <h3 class={styles.subtitle}>Room settings</h3>
              <label class={styles.text_input}>
                <div>
                  <FaSolidUser />
                  <input placeholder="Write your name" />
                </div>
              </label>

              <label class={styles.checkbox_input}>
                Public room
                <input type="checkbox" />
              </label>

              <label class={styles.text_input}>
                <div>
                  <FaSolidLock />
                  <input placeholder="Leave empty for no password" />
                </div>
              </label>
            </div>

            <div>
              <h3 class={styles.subtitle}>Members</h3>
              <label class={styles.text_input}>
                <div>
                  <FaBrandsGithub />
                  <input placeholder="Username" />
                </div>
              </label>
              <ul class={styles.user_list}>
                <For each={requestUsers}>
                  {(name, idx) => (
                    <li class={styles.member}>
                      <span class={styles.member_name}>
                        {name}
                      </span>

                      <Popover placement="right-start">
                        <Popover.Trigger class={styles.select_box}>
                          <span>{idx() % 2 === 0 ? "Editor" : "Viewer"}</span>
                          <div>
                            <FaSolidChevronDown width="0.5em" height="0.5em" />
                          </div>
                        </Popover.Trigger>
                        <Popover.Portal>
                          <Popover.Content as="ul" class={styles.select_list}>
                            <label>
                              <input
                                type="radio"
                                name="select"
                                checked={idx() % 2 === 0}
                              />
                              <span>Editor</span>
                            </label>
                            <label>
                              <input
                                type="radio"
                                name="select"
                                checked={idx() % 2 !== 0}
                              />
                              <span>Viewer</span>
                            </label>
                          </Popover.Content>
                        </Popover.Portal>
                      </Popover>
                    </li>
                  )}
                </For>
              </ul>

              <h3 class={styles.subtitle}>Pending Requests</h3>
              <ul class={styles.user_list}>
                <For each={requestUsers}>
                  {(name) => (
                    <li class={styles.member}>
                      <span class={styles.member_name}>
                        {name}
                      </span>

                      <ul class={styles.button_group}>
                        <button class={styles.success}>Allow</button>
                        <button class={styles.error}>Kick</button>
                      </ul>
                    </li>
                  )}
                </For>
              </ul>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog>
  );
}
