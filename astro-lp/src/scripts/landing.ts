function asElement(node: Element | null): HTMLElement | null {
  return node instanceof HTMLElement ? node : null;
}

function revealOnScroll(): void {
  const staggered = document.querySelectorAll<HTMLElement>("[data-stagger]");
  staggered.forEach((element, index) => {
    const slot = index % 3;
    if (slot === 1) element.dataset.delay = "1";
    if (slot === 2) element.dataset.delay = "2";
  });

  const targets = document.querySelectorAll<HTMLElement>("[data-reveal]");
  if (!("IntersectionObserver" in window)) {
    targets.forEach((element) => {
      element.dataset.in = "true";
    });
    return;
  }

  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting || !(entry.target instanceof HTMLElement)) return;
        entry.target.dataset.in = "true";
        observer.unobserve(entry.target);
      });
    },
    { threshold: 0.15 }
  );
  for (const element of targets) observer.observe(element);
}

function bindScrollChrome(): void {
  const progress = document.getElementById("prog");
  const header = asElement(document.querySelector("[data-header]"));
  const hero = asElement(document.querySelector("[data-hero]"));
  const stage = asElement(document.querySelector("[data-stage]"));
  const root = document.documentElement;

  const onScroll = (): void => {
    if (progress) {
      const max = root.scrollHeight - root.clientHeight;
      progress.style.width = `${max > 0 ? (root.scrollTop / max) * 100 : 0}%`;
    }
    if (header) {
      if (root.scrollTop > 10) header.dataset.scrolled = "true";
      else delete header.dataset.scrolled;
    }
  };

  window.addEventListener("scroll", onScroll, { passive: true });
  onScroll();

  if (hero && window.matchMedia("(hover: hover)").matches) {
    hero.addEventListener("mousemove", (event) => {
      const bounds = hero.getBoundingClientRect();
      hero.style.setProperty("--mx", `${event.clientX - bounds.left}px`);
      hero.style.setProperty("--my", `${event.clientY - bounds.top}px`);
      if (!stage) return;
      const x = event.clientX / window.innerWidth - 0.5;
      const y = event.clientY / window.innerHeight - 0.5;
      stage.style.transform = `rotateY(${x * 8}deg) rotateX(${-y * 6}deg)`;
    });
    hero.addEventListener("mouseleave", () => {
      if (stage) stage.style.transform = "";
    });
  }

  let ticking = false;
  window.addEventListener(
    "scroll",
    () => {
      if (ticking) return;
      ticking = true;
      requestAnimationFrame(() => {
        root.style.setProperty("--sy", String(window.scrollY));
        ticking = false;
      });
    },
    { passive: true }
  );
}

function bindTilt(): void {
  if (!window.matchMedia("(hover: hover)").matches) return;
  document.querySelectorAll<HTMLElement>("[data-tilt]").forEach((card) => {
    card.addEventListener("mousemove", (event) => {
      const bounds = card.getBoundingClientRect();
      card.style.setProperty("--sx", `${event.clientX - bounds.left}px`);
      card.style.setProperty("--sy", `${event.clientY - bounds.top}px`);
      const x = (event.clientX - bounds.left) / bounds.width - 0.5;
      const y = (event.clientY - bounds.top) / bounds.height - 0.5;
      card.style.transform = `perspective(900px) rotateX(${-y * 7}deg) rotateY(${x * 9}deg) translateY(-6px)`;
    });
    card.addEventListener("mouseleave", () => {
      card.style.transform = "";
    });
  });
}

function bindDemo(): void {
  const ime = document.getElementById("ime");
  const pointer = document.getElementById("ptr");
  const toast = document.getElementById("toast");
  const caption = document.getElementById("cap");
  const chatText = document.getElementById("tChat");
  const codeText = document.getElementById("tCode");
  const chat = document.getElementById("pChat");
  const code = document.getElementById("pCode");
  const play = document.getElementById("dplay");
  const demo = document.getElementById("demo");
  const timeline = document.getElementById("tline");
  if (
    !ime ||
    !pointer ||
    !toast ||
    !caption ||
    !chatText ||
    !codeText ||
    !chat ||
    !code ||
    !play ||
    !demo ||
    !timeline
  ) {
    return;
  }

  let paused = false;
  let run = 0;
  const wait = (ms: number): Promise<void> =>
    new Promise((resolve) => {
      window.setTimeout(resolve, ms);
    });

  const sleep = async (ms: number): Promise<void> => {
    let elapsed = 0;
    while (elapsed < ms) {
      if (paused) {
        await wait(120);
        continue;
      }
      await wait(40);
      elapsed += 40;
    }
  };

  const type = async (element: HTMLElement, text: string, speed: number): Promise<void> => {
    for (const char of text) {
      element.textContent = `${element.textContent ?? ""}${char}`;
      await sleep(speed);
    }
  };

  const move = (x: number, y: number): void => {
    pointer.style.left = `${x}%`;
    pointer.style.top = `${y}%`;
  };

  const click = (): void => {
    pointer.dataset.click = "false";
    void pointer.offsetWidth;
    pointer.dataset.click = "true";
  };

  const setIme = (value: string, message: string): void => {
    ime.textContent = value;
    ime.dataset.ime = value === "あ" ? "ja" : "en";
    ime.dataset.flash = "true";
    window.setTimeout(() => {
      delete ime.dataset.flash;
    }, 500);
    toast.textContent = message;
    toast.dataset.show = "true";
    window.setTimeout(() => {
      delete toast.dataset.show;
    }, 1700);
  };

  const markStep = (step: number, label: string): void => {
    caption.textContent = label;
    const fill = step === 2 ? "0.5" : step === 3 ? "1" : "0";
    timeline.style.setProperty("--f", fill);
    document.querySelectorAll<HTMLElement>("[data-demo-step]").forEach((element) => {
      const index = Number(element.dataset.demoStep);
      if (index === step) element.dataset.state = "on";
      else if (index < step) element.dataset.state = "done";
      else delete element.dataset.state;
    });
  };

  const loop = async (): Promise<void> => {
    const id = ++run;
    while (id === run) {
      chatText.textContent = "";
      codeText.textContent = "";
      delete chat.dataset.active;
      code.dataset.active = "true";
      ime.textContent = "A";
      ime.dataset.ime = "en";
      move(88, 92);
      markStep(0, "エディタでコードを書いている途中…");
      await sleep(1300);
      if (id !== run) return;
      markStep(1, "① チャット欄をクリック");
      move(24, 76);
      await sleep(1100);
      click();
      delete code.dataset.active;
      chat.dataset.active = "true";
      setIme("あ", "日本語に自動で切り替え");
      await sleep(900);
      markStep(2, "② そのまま日本語で入力");
      await type(chatText, "このコードの意味を教えて", 90);
      await sleep(900);
      markStep(3, "③ コードエディタをクリック");
      move(70, 48);
      await sleep(1100);
      click();
      delete chat.dataset.active;
      code.dataset.active = "true";
      setIme("A", "英語に自動で切り替え");
      await sleep(700);
      await type(codeText, 'greet("World");', 90);
      await sleep(2200);
    }
  };

  play.addEventListener("click", () => {
    paused = !paused;
    play.textContent = paused ? "▶" : "⏸";
    play.setAttribute("aria-label", paused ? "再生" : "一時停止");
  });

  let started = false;
  new IntersectionObserver(
    (entries) => {
      const entry = entries[0];
      if (!entry?.isIntersecting || started) return;
      started = true;
      void loop();
    },
    { threshold: 0.3 }
  ).observe(demo);
}

function bindHow(): void {
  const section = document.getElementById("how");
  const track = asElement(section?.querySelector("[data-how-track]") ?? null);
  if (!section || !track) return;
  const items = section.querySelectorAll<HTMLElement>("[data-how-step]");

  const update = (): void => {
    const bounds = track.getBoundingClientRect();
    const total = bounds.height - window.innerHeight;
    const progress = total > 0 ? Math.min(1, Math.max(0, -bounds.top / total)) : 0;
    const step = progress < 0.34 ? 0 : progress < 0.67 ? 1 : 2;
    section.style.setProperty("--lp", String(Math.min(1, Math.max(0, progress * 3 - step))));
    section.dataset.step = String(step);
    items.forEach((item, index) => {
      if (index < step) item.dataset.state = "act";
      else if (index === step) item.dataset.state = "cur";
      else delete item.dataset.state;
    });
  };

  window.addEventListener("scroll", update, { passive: true });
  window.addEventListener("resize", update);
  update();

  items.forEach((item, index) => {
    item.addEventListener("click", () => {
      const bounds = track.getBoundingClientRect();
      const total = bounds.height - window.innerHeight;
      const top = window.scrollY + bounds.top;
      window.scrollTo({ top: top + total * ((index + 0.5) / 3), behavior: "smooth" });
    });
  });
}

function bindSideNav(): void {
  const links = document.querySelectorAll<HTMLAnchorElement>("[data-side-link]");
  const sections = [...links]
    .map((link) => {
      const id = link.dataset.section;
      const section = id ? document.getElementById(id) : null;
      return section ? { section, link } : null;
    })
    .filter((item): item is { section: HTMLElement; link: HTMLAnchorElement } => item !== null);
  const cta = asElement(document.querySelector("[data-floating]"));
  const download = document.getElementById("download");

  const update = (): void => {
    const mid = window.innerHeight * 0.45;
    let current = -1;
    sections.forEach((item, index) => {
      const bounds = item.section.getBoundingClientRect();
      if (bounds.top <= mid && bounds.bottom > mid) current = index;
    });
    sections.forEach((item, index) => {
      if (index === current) item.link.dataset.current = "true";
      else delete item.link.dataset.current;
    });
    const downloadTop = download?.getBoundingClientRect().top;
    const hide = downloadTop !== undefined && downloadTop < window.innerHeight * 0.9;
    if (cta) cta.dataset.show = window.scrollY > 650 && !hide ? "true" : "false";
  };

  window.addEventListener("scroll", update, { passive: true });
  window.addEventListener("resize", update);
  update();
}

export function startLanding(): void {
  revealOnScroll();
  bindScrollChrome();
  bindTilt();
  bindDemo();
  bindHow();
  bindSideNav();
}
