(() => {
  const root = document.getElementById('project-printer');
  const printer = root.querySelector('.printer');
  const paperWindow = root.querySelector('.paper-window');
  const outputSlot = root.querySelector('.output-slot');
  const headTrack = root.querySelector('.head-track');
  const writerHead = root.querySelector('.writer-head');
  const status = root.querySelector('.status span');
  const nextButton = root.querySelector('.feed');
  const motionPreference = window.matchMedia('(prefers-reduced-motion: reduce)');
  let idleTimer = 0;

  function atEnd() {
    return paperWindow.scrollTop + paperWindow.clientHeight >= paperWindow.scrollHeight - 2;
  }

  function rest() {
    printer.classList.remove('printing');
    status.textContent = atEnd() ? printer.dataset.end : printer.dataset.ready;
    nextButton.disabled = atEnd();
  }

  paperWindow.addEventListener('scroll', () => {
    if (!printer.classList.contains('printing')) {
      printer.classList.add('printing');
      status.textContent = printer.dataset.printing;
    }
    clearTimeout(idleTimer);
    idleTimer = setTimeout(rest, 180);
  }, { passive: true });

  // Wheel gestures over the fixed case should still feed the paper. Gestures
  // inside the printout (including horizontally scrolling tables) stay native.
  document.addEventListener('wheel', event => {
    if (paperWindow.contains(event.target) || event.ctrlKey || !event.deltaY) return;
    const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? paperWindow.clientHeight : 1;
    paperWindow.scrollBy({ top: event.deltaY * unit, behavior: 'instant' });
    event.preventDefault();
  }, { passive: false });

  nextButton.addEventListener('click', () => {
    const origin = paperWindow.getBoundingClientRect().top;
    const next = Array.from(root.querySelectorAll('.paper-entry, .end-note'))
      .map(entry => entry.getBoundingClientRect().top - origin + paperWindow.scrollTop)
      .find(top => top > paperWindow.scrollTop + 30);
    paperWindow.scrollTo({
      top: next === undefined ? paperWindow.scrollHeight : Math.max(0, next - 12),
      behavior: motionPreference.matches ? 'instant' : 'smooth',
    });
  });

  function measurePrinter() {
    const exit = outputSlot.getBoundingClientRect().top;
    paperWindow.style.setProperty('--paper-exit-y', `${exit}px`);
    root.style.setProperty('--printer-clearance', `${Math.ceil(window.innerHeight - exit) + 60}px`);
    root.style.setProperty('--head-travel', `${Math.max(0, headTrack.clientWidth - writerHead.offsetWidth)}px`);
    rest();
  }

  nextButton.hidden = false;
  new ResizeObserver(measurePrinter).observe(printer);
  window.addEventListener('resize', measurePrinter, { passive: true });
  measurePrinter();
})();
