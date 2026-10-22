(() => {
  const root = document.getElementById('project-printer');
  const printer = root.querySelector('.printer');
  const headTrack = root.querySelector('.mechanism');
  const printHead = root.querySelector('.print-head');
  const paperWindow = root.querySelector('.paper-window');
  const outputSlot = root.querySelector('.output-slot');
  const status = root.querySelector('.status span');
  const nextButton = root.querySelector('.feed');
  const motionPreference = window.matchMedia('(prefers-reduced-motion: reduce)');
  let frame = 0;
  let idleTimer = 0;

  function atEnd() {
    return window.scrollY + window.innerHeight >= document.documentElement.scrollHeight - 2;
  }

  function rest() {
    printer.classList.remove('printing');
    status.textContent = atEnd() ? printer.dataset.end : printer.dataset.ready;
    nextButton.disabled = atEnd();
  }

  function clipPaper() {
    // Clip at the fixed paper exit, including when motion is disabled. The
    // unprinted part of the document must never appear below the machine.
    const printLine = outputSlot.getBoundingClientRect().top + 7;
    const hidden = Math.max(0, paperWindow.getBoundingClientRect().bottom - printLine);
    paperWindow.style.setProperty('--paper-hidden', `${hidden}px`);
  }

  function updatePrintFrame() {
    frame = 0;
    clipPaper();
    if (motionPreference.matches) return;
    const distance = Math.max(0, headTrack.clientWidth - printHead.offsetWidth - 10);
    const phase = (window.scrollY % 200) / 100;
    const position = phase <= 1 ? phase : 2 - phase;
    root.style.setProperty('--head', `${5 + position * distance}px`);
    root.style.setProperty('--roller', `${-window.scrollY % 60}px`);
  }

  window.addEventListener('scroll', () => {
    printer.classList.add('printing');
    status.textContent = printer.dataset.printing;
    if (!frame) frame = requestAnimationFrame(updatePrintFrame);
    clearTimeout(idleTimer);
    idleTimer = setTimeout(rest, 180);
  }, { passive: true });

  nextButton.addEventListener('click', () => {
    const next = Array.from(root.querySelectorAll('.paper-entry, .end-note'))
      .map(entry => entry.getBoundingClientRect().top + window.scrollY)
      .find(top => top > window.scrollY + 30);
    window.scrollTo({
      top: next === undefined ? document.documentElement.scrollHeight : Math.max(0, next - 12),
      behavior: motionPreference.matches ? 'instant' : 'smooth',
    });
  });

  function measurePrinter() {
    const clearance = `${Math.ceil(printer.getBoundingClientRect().height) + 60}px`;
    root.style.setProperty('--printer-clearance', clearance);
    document.documentElement.style.setProperty('--printer-clearance', clearance);
    updatePrintFrame();
    rest();
  }

  nextButton.hidden = false;
  new ResizeObserver(measurePrinter).observe(printer);
  new ResizeObserver(clipPaper).observe(paperWindow);
  window.addEventListener('resize', measurePrinter, { passive: true });
  measurePrinter();
})();
