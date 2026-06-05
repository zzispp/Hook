const root = document.documentElement;
const storedTheme = localStorage.getItem('hook-docs-theme');
const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
root.dataset.theme = storedTheme || (prefersDark ? 'dark' : 'light');

document.querySelectorAll('[data-theme-toggle]').forEach((button) => {
  button.addEventListener('click', () => {
    const nextTheme = root.dataset.theme === 'dark' ? 'light' : 'dark';
    root.dataset.theme = nextTheme;
    localStorage.setItem('hook-docs-theme', nextTheme);
  });
});

const copyButtons = document.querySelectorAll('[data-copy]');

const copyLabels = (button) => {
  const idle = button.dataset.idleText || button.textContent.trim();
  button.dataset.idleText = idle;
  return idle === '复制'
    ? { idle, success: '已复制', failure: '失败' }
    : { idle, success: 'Copied', failure: 'Failed' };
};

const setCopyState = (button, text, className, idleText) => {
  button.classList.remove('done', 'failed');
  button.classList.add(className);
  button.textContent = text;
  window.setTimeout(() => {
    button.classList.remove(className);
    button.textContent = idleText;
  }, 1600);
};

copyButtons.forEach((button) => {
  button.addEventListener('click', async () => {
    const target = document.querySelector(button.dataset.copy);
    if (!target) {
      throw new Error(`Copy target not found: ${button.dataset.copy}`);
    }

    const labels = copyLabels(button);
    try {
      await navigator.clipboard.writeText(target.textContent.trim());
      setCopyState(button, labels.success, 'done', labels.idle);
    } catch (error) {
      console.error(error);
      setCopyState(button, labels.failure, 'failed', labels.idle);
    }
  });
});

const navLinks = [...document.querySelectorAll('.topnav a, .sidebar a')];
const sections = [...document.querySelectorAll('.doc-section')];

const observer = new IntersectionObserver(
  (entries) => {
    const visible = entries
      .filter((entry) => entry.isIntersecting)
      .sort((a, b) => b.intersectionRatio - a.intersectionRatio)[0];

    if (!visible) {
      return;
    }

    navLinks.forEach((link) => {
      link.classList.toggle('active', link.getAttribute('href') === `#${visible.target.id}`);
    });
  },
  { rootMargin: '-25% 0px -55% 0px', threshold: [0.1, 0.3, 0.6] }
);

sections.forEach((section) => observer.observe(section));

const dialog = document.querySelector('[data-search-dialog]');
const searchInput = document.querySelector('[data-search-input]');
const searchResults = document.querySelector('[data-search-results]');
const searchItems = sections.map((section) => ({
  id: section.id,
  title: section.querySelector('h2')?.textContent.trim() || section.id,
  text: section.textContent.replace(/\s+/g, ' ').trim(),
}));

const searchLabels = document.documentElement.lang.startsWith('zh')
  ? { empty: '没有匹配的章节' }
  : { empty: 'No matching sections' };

const renderResults = (query = '') => {
  const normalized = query.trim().toLowerCase();
  const matches = normalized
    ? searchItems.filter((item) => item.text.toLowerCase().includes(normalized))
    : searchItems;

  searchResults.innerHTML = '';
  const visible = matches.slice(0, 8);
  if (!visible.length) {
    searchResults.innerHTML = `<p class="search-result">${searchLabels.empty}</p>`;
    return;
  }

  visible.forEach((item) => {
    const link = document.createElement('a');
    link.className = 'search-result';
    link.href = `#${item.id}`;
    const title = document.createElement('strong');
    title.textContent = item.title;
    link.append(title);
    link.addEventListener('click', () => dialog.close());
    searchResults.append(link);
  });
};

const openSearch = () => {
  if (!dialog || !dialog.showModal) {
    throw new Error('Search dialog is not supported by this browser.');
  }

  renderResults(searchInput.value);
  dialog.showModal();
  searchInput.focus();
};

document.querySelectorAll('[data-search-open]').forEach((button) => {
  button.addEventListener('click', openSearch);
});

document.querySelectorAll('[data-search-close]').forEach((button) => {
  button.addEventListener('click', () => dialog.close());
});

searchInput?.addEventListener('input', () => renderResults(searchInput.value));

document.addEventListener('keydown', (event) => {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault();
    openSearch();
  }
});

renderResults();
