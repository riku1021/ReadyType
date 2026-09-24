import { Children, isValidElement, type ReactNode } from "react";

import * as styles from "./SettingsSection.styles";

type SettingsSectionProps = {
  title?: string;
  children: ReactNode;
};

function getItemKey(child: ReactNode, index: number): string {
  if (isValidElement(child) && child.key != null) {
    return String(child.key);
  }
  return `settings-item-${index}`;
}

export function SettingsSection({ title, children }: SettingsSectionProps) {
  const items = Children.toArray(children).filter(Boolean);

  return (
    <section className={styles.root}>
      {title ? <h2 className={styles.title}>{title}</h2> : null}
      <div className={styles.card}>
        {items.flatMap((child, index) => {
          if (index === 0) {
            return [child];
          }
          const key = getItemKey(child, index);
          return [<hr key={`divider-${key}`} className={styles.divider} />, child];
        })}
      </div>
    </section>
  );
}
