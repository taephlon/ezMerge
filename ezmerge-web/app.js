// ==========================================================================
// ezMerge Frontend Application Logic
// ==========================================================================

let appDatabase = { packages: [], overlays: [] };
let selectedPackage = null;
let currentTab = 'packages';

// Pagination variables
let currentPackages = [];
let visibleCount = 8;
let searchTimeout = null;

// Fetch package database
document.addEventListener('DOMContentLoaded', () => {
  initTabs();
  fetchDatabase();
  setupSearch();
  
  // Bind click event to Show More button
  const btnShowMore = document.getElementById('btn-show-more');
  if (btnShowMore) {
    btnShowMore.addEventListener('click', () => {
      visibleCount += 8;
      renderPackages(currentPackages);
    });
  }
});

// Fetch database json on start
async function fetchDatabase() {
  const container = document.getElementById('package-list-container');
  try {
    // Try to load db.json from local or absolute paths
    let response = await fetch('../ezmerge-api/db.json');
    if (!response.ok) {
      response = await fetch('db.json');
    }
    if (!response.ok) {
      throw new Error(`Failed to load database: ${response.statusText}`);
    }
    appDatabase = await response.json();
    
    renderPackages(appDatabase.packages);
    renderOverlays(appDatabase.overlays);
  } catch (error) {
    console.error('Error fetching package database:', error);
    if (container) {
      container.innerHTML = `
        <div style="color: var(--red-alert); padding: 20px; border: 1px solid var(--red-alert); border-radius: 8px; background: rgba(248,113,113,0.05);">
          <strong>⚠️ Failed to load database:</strong> ${error.message}<br>
          Make sure the ezmerge-api server is running or db.json is accessible.
        </div>
      `;
    }
  }
}

// Render Package Cards List (Minimized and Paginated)
fnRenderPackages = function(packages) {
  currentPackages = packages;
  const listContainer = document.getElementById('package-list-container');
  const paginationContainer = document.getElementById('pagination-container');
  const btnShowMore = document.getElementById('btn-show-more');
  
  if (!listContainer) return;
  listContainer.innerHTML = '';

  if (packages.length === 0) {
    listContainer.innerHTML = '<div class="empty-state">No packages found matching search criteria.</div>';
    if (paginationContainer) paginationContainer.classList.add('hidden');
    return;
  }

  // Slice list to visibleCount
  const slice = packages.slice(0, visibleCount);

  slice.forEach(pkg => {
    const card = document.createElement('div');
    card.className = 'package-card';
    card.style.padding = '10px 14px';
    card.style.borderRadius = '6px';
    card.style.marginBottom = '8px';
    
    if (selectedPackage && selectedPackage.atom === pkg.atom) {
      card.classList.add('selected-card');
    }

    const maskedBadge = pkg.masked ? `<span class="mask-badge" style="font-size: 0.75rem; padding: 1px 5px;">masked</span>` : '';
    
    // Minimized HTML card structure (removed description to save height space)
    card.innerHTML = `
      <div class="card-header" style="margin-bottom:0; display:flex; justify-content:space-between; align-items:center;">
        <span class="pkg-atom" style="font-size:0.95rem; font-family:var(--font-mono); font-weight:700;">${pkg.atom}</span>
        <span class="pkg-ver" style="font-size:0.8rem; padding: 1px 5px;">${pkg.version}</span>
      </div>
      <div class="card-footer" style="margin-top:6px; display:flex; justify-content:space-between; align-items:center;">
        ${pkg.overlay === 'gentoo' 
          ? `<span class="overlay-badge" style="background: rgba(52, 211, 153, 0.1); color: var(--green-success); border-color: rgba(52, 211, 153, 0.2); font-size: 0.75rem; padding: 1px 5px;">gentoo (official)</span>`
          : `<span class="overlay-badge" style="font-size: 0.75rem; padding: 1px 5px;">@${pkg.overlay}</span>`
        }
        ${maskedBadge}
      </div>
    `;

    card.addEventListener('click', () => {
      // Toggle selected class
      document.querySelectorAll('.package-card').forEach(c => c.classList.remove('selected-card'));
      card.classList.add('selected-card');
      selectPackage(pkg);
    });

    listContainer.appendChild(card);
  });

  // Handle "Show more" button pagination display
  if (paginationContainer && btnShowMore) {
    if (packages.length > visibleCount) {
      paginationContainer.classList.remove('hidden');
      btnShowMore.innerText = `Show more packages (${packages.length - visibleCount} remaining)`;
    } else {
      paginationContainer.classList.add('hidden');
    }
  }
}
window.renderPackages = fnRenderPackages;

// Select Package and Render Details Card
function selectPackage(pkg) {
  selectedPackage = pkg;
  const container = document.getElementById('package-details-container');
  if (!container) return;

  // Find overlay details
  const overlay = appDatabase.overlays.find(o => o.name === pkg.overlay);
  let starsHtml = '';
  if (overlay) {
    const stars = Math.min(Math.round(overlay.trust_score), 5);
    starsHtml = `<span class="overlay-stars" title="Trust Score: ${overlay.trust_score}">` + '★'.repeat(stars) + '☆'.repeat(5 - stars) + '</span>';
  }

  // Keywords list
  let keywordsHtml = '';
  if (pkg.keywords && pkg.keywords.length > 0) {
    keywordsHtml = pkg.keywords.map(kw => {
      const isTesting = kw.startsWith('~');
      return `<span class="meta-pill" style="border-color: ${isTesting ? 'var(--yellow-warning)' : 'var(--green-success)'}; color: ${isTesting ? 'var(--yellow-warning)' : 'var(--green-success)'}">${kw}</span>`;
    }).join(' ');
  } else {
    keywordsHtml = `<span class="meta-pill" style="border-color: var(--green-success); color: var(--green-success)">stable</span>`;
  }

  // Dependencies list
  let depsHtml = '<li>None</li>';
  if (pkg.dependencies && pkg.dependencies.length > 0) {
    depsHtml = pkg.dependencies.map(dep => `<li class="dep-item">${dep}</li>`).join('');
  }

  // USE Flags rows
  let useFlagsRows = '<tr><td colspan="3" style="text-align:center; color:var(--text-muted);">No USE flags available.</td></tr>';
  if (pkg.use_flags && pkg.use_flags.length > 0) {
    useFlagsRows = pkg.use_flags.map((flag, idx) => {
      return `
        <tr>
          <td>
            <div class="flag-toggle-wrapper">
              <input type="checkbox" id="flag-chk-${idx}" class="flag-checkbox" ${flag.default ? 'checked' : ''} onchange="updateInstallCommand()">
              <label for="flag-chk-${idx}" id="flag-label-${idx}" class="flag-name ${flag.default ? 'enabled-flag' : 'disabled-flag'}">${flag.name}</label>
            </div>
          </td>
          <td><span style="color: ${flag.default ? 'var(--green-success)' : 'var(--text-muted)'}; font-weight: 500;">${flag.default ? 'Enabled' : 'Disabled'}</span></td>
          <td class="flag-desc">${flag.description}</td>
        </tr>
      `;
    }).join('');
  }

  // Render Card Content
  container.innerHTML = `
    <div class="detail-title-row">
      <h3 class="detail-atom">${pkg.atom}</h3>
      <div class="detail-meta-flex">
        <span class="meta-pill"><span class="meta-pill-label">License:</span>${pkg.license}</span>
        <span class="meta-pill"><span class="meta-pill-label">Version:</span>${pkg.version}</span>
        ${keywordsHtml}
      </div>
    </div>

    ${pkg.masked ? `
      <div style="background: rgba(248,113,113,0.08); border: 1px solid rgba(248,113,113,0.2); border-radius: 8px; padding: 12px; margin-bottom: 20px; color: var(--red-alert);">
        <strong style="display:block; margin-bottom: 4px;">⚠️ Masked Package</strong>
        <p style="font-size: 0.85rem; font-style: italic;">Reason: ${pkg.mask_reason || 'Ebuild is masked in repository.'}</p>
      </div>
    ` : ''}

    <p class="detail-desc">${pkg.description}</p>

    <div class="detail-section-title">Overlay Source</div>
    <div style="margin-bottom: 15px;">
      <strong style="font-size:1.05rem;">@${pkg.overlay}</strong> ${starsHtml}
      ${overlay ? `<p style="font-size:0.85rem; color:var(--text-dimmed); margin-top:4px;">${overlay.description}</p>` : ''}
      ${pkg.homepage ? `<a href="${pkg.homepage}" target="_blank" style="display:inline-block; font-size:0.85rem; margin-top:8px; text-decoration:underline;">Visit Project Homepage ↗</a>` : ''}
    </div>

    <div class="detail-section-title">Dependencies</div>
    <ul class="dep-tree">
      ${depsHtml}
    </ul>

    <div class="detail-section-title">Configure USE Flags</div>
    <div class="flags-table-container">
      <table class="flags-table">
        <thead>
          <tr>
            <th>Flag Name</th>
            <th>Default</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          ${useFlagsRows}
        </tbody>
      </table>
    </div>

    <div class="detail-section-title">Emerge Preview (emerge -av)</div>
    <div class="install-command-block" style="flex-direction: column; align-items: stretch; font-family: var(--font-mono); font-size: 0.85rem; line-height: 1.6; padding: 16px; background: #060913; border: 1px solid var(--border-color-active); border-radius: 8px; box-shadow: inset 0 2px 10px rgba(0, 0, 0, 0.5);">
      <div style="color: var(--text-muted); margin-bottom: 8px; font-style: italic;">These are the packages that would be merged, in order:</div>
      <div id="emerge-preview-deps" style="white-space: pre-wrap; margin-bottom: 4px;"></div>
      <div id="emerge-preview-main" style="white-space: pre-wrap; margin-bottom: 8px;"></div>
      <div style="border-top: 1px solid var(--border-color); margin-top: 10px; padding-top: 8px; color: var(--text-dimmed);" id="emerge-preview-summary"></div>
    </div>

    <div class="detail-section-title">Install Command</div>
    <div class="install-command-block">
      <span class="cmd-text" id="install-cmd-snippet">ezmerge install ${pkg.name}</span>
      <button class="copy-btn" id="btn-copy-install-cmd" onclick="copyInstallCmd()">Copy</button>
    </div>
  `;

  updateInstallCommand();
}

// Update the install command text and live emerge -av preview based on flag checkboxes
window.updateInstallCommand = function() {
  if (!selectedPackage) return;
  const snippet = document.getElementById('install-cmd-snippet');
  if (!snippet) return;

  const flagCheckboxes = document.querySelectorAll('.flag-checkbox');
  const activeFlags = [];
  const disabledFlags = [];

  flagCheckboxes.forEach((chk, idx) => {
    const label = document.getElementById(`flag-label-${idx}`);
    const flag = selectedPackage.use_flags[idx];
    if (chk.checked) {
      activeFlags.push(flag.name);
      if (label) {
        label.classList.add('enabled-flag');
        label.classList.remove('disabled-flag');
      }
    } else {
      disabledFlags.push(`-${flag.name}`);
      if (label) {
        label.classList.remove('enabled-flag');
        label.classList.add('disabled-flag');
      }
    }
  });

  snippet.innerText = `ezmerge install ${selectedPackage.name}`;

  // Live emerge -av simulation blocks
  const previewDeps = document.getElementById('emerge-preview-deps');
  const previewMain = document.getElementById('emerge-preview-main');
  const previewSummary = document.getElementById('emerge-preview-summary');

  if (previewDeps && previewMain && previewSummary) {
    let depsHtml = '';
    let totalPackages = 1;
    let newPackages = 1;
    let rebuildPackages = 0;
    let totalSize = (selectedPackage.name.length * 123) % 2000 + 200;

    // Render dependency lines
    if (selectedPackage.dependencies && selectedPackage.dependencies.length > 0) {
      selectedPackage.dependencies.forEach(dep => {
        totalPackages += 1;
        newPackages += 1;
        const depSize = (dep.length * 17) % 500 + 50;
        totalSize += depSize;
        depsHtml += ` <span style="color: var(--green-success)">[ebuild  N     ]</span> <span style="font-weight: bold; color: var(--text-main);">${dep}</span>::gentoo <span style="color: var(--text-muted); font-size:0.8rem;">[${depSize} KiB]</span>\n`;
      });
    }
    previewDeps.innerHTML = depsHtml;

    // Build colored USE flags string
    let useStr = '';
    if (selectedPackage.use_flags && selectedPackage.use_flags.length > 0) {
      useStr = ' <span style="color: var(--text-muted)">USE="</span>';
      const flagParts = [];
      selectedPackage.use_flags.forEach((flag, idx) => {
        const chk = document.getElementById(`flag-chk-${idx}`);
        if (chk && chk.checked) {
          // Enabled flags: bold red (Portage style)
          flagParts.push(`<span style="color: var(--red-alert); font-weight: bold;">${flag.name}</span>`);
        } else {
          // Disabled flags: blue with minus prefix
          flagParts.push(`<span style="color: #60a5fa; font-weight: normal;">-${flag.name}</span>`);
        }
      });
      useStr += flagParts.join(' ') + '<span style="color: var(--text-muted)">"</span>';
    }

    // Render target package line
    const mainSize = (selectedPackage.name.length * 123) % 2000 + 200;
    const isMainInstalled = selectedPackage.name === 'zsh' || selectedPackage.name === 'neovim' || selectedPackage.name === 'portage';
    const statusType = isMainInstalled 
      ? `<span style="color: var(--yellow-warning)">[ebuild   R    ]</span>`
      : `<span style="color: var(--green-success)">[ebuild  N     ]</span>`;
      
    if (isMainInstalled) {
      rebuildPackages += 1;
    } else {
      newPackages += 1;
    }

    previewMain.innerHTML = ` ${statusType} <span style="font-weight: bold; color: var(--text-main);">${selectedPackage.atom}</span>::${selectedPackage.overlay} ${useStr} <span style="color: var(--text-muted); font-size:0.8rem;">[${mainSize} KiB]</span>`;

    // Render totals summary line
    previewSummary.innerHTML = `Total: <span style="color: var(--text-main); font-weight: 600;">${totalPackages} packages</span> (${newPackages} new, ${rebuildPackages} rebuild), Size of downloads: <span style="color: var(--green-success); font-weight:600;">${totalSize} KiB</span>`;
  }
}


// Copy Install command
window.copyInstallCmd = function() {
  const snippet = document.getElementById('install-cmd-snippet');
  if (!snippet) return;
  copyText(snippet.innerText, 'btn-copy-install-cmd');
}

// Copy Helper
window.copyText = function(text, buttonId) {
  navigator.clipboard.writeText(text).then(() => {
    const button = document.getElementById(buttonId);
    if (!button) return;
    const oldText = button.innerText;
    button.innerText = 'Copied!';
    button.style.background = 'var(--green-success)';
    button.style.color = '#0b0f19';
    setTimeout(() => {
      button.innerText = oldText;
      button.style.background = '';
      button.style.color = '';
    }, 2000);
  }).catch(err => {
    console.error('Failed to copy text:', err);
  });
}

// Setup live package search with debounce and real-time backend API crawler
function setupSearch() {
  const searchInput = document.getElementById('package-search-input');
  if (!searchInput) return;

  searchInput.addEventListener('input', (e) => {
    const query = e.target.value.trim();
    
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }

    searchTimeout = setTimeout(async () => {
      visibleCount = 8; // Reset visible count on new query
      
      if (query === '') {
        renderPackages(appDatabase.packages);
        return;
      }

      try {
        // Query the custom python backend repository ebuild crawler
        const response = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
        if (!response.ok) {
          throw new Error('API server returned error');
        }
        const results = await response.json();
        renderPackages(results);
      } catch (err) {
        console.warn('API search failed, falling back to static database filtering:', err);
        // Fallback filter locally in case server-less
        const query_lower = query.toLowerCase();
        const filtered = appDatabase.packages.filter(pkg => {
          return pkg.name.toLowerCase().includes(query_lower) ||
                 pkg.atom.toLowerCase().includes(query_lower) ||
                 pkg.description.toLowerCase().includes(query_lower) ||
                 pkg.overlay.toLowerCase().includes(query_lower);
        });
        renderPackages(filtered);
      }
    }, 250); // 250ms debounce time
  });
}

// Render Overlay Cards List
function renderOverlays(overlays) {
  const container = document.getElementById('overlays-container');
  if (!container) return;
  container.innerHTML = '';

  overlays.forEach(overlay => {
    const stars = Math.min(Math.round(overlay.trust_score), 5);
    const starHtml = '★'.repeat(stars) + '☆'.repeat(5 - stars);

    const card = document.createElement('div');
    card.className = 'overlay-card';
    card.innerHTML = `
      <div class="overlay-card-header">
        <span class="overlay-name">@${overlay.name}</span>
        <span class="overlay-stars">${starHtml}</span>
      </div>
      <span class="overlay-url">${overlay.url}</span>
      <p class="overlay-description">${overlay.description}</p>
      <div class="overlay-stats-row">
        <span>Packages: <span class="stat-val">${overlay.packages_count}</span></span>
        <span>Maintainers: <span class="stat-val">${overlay.maintainers}</span></span>
        <span>Sync: <span class="stat-val">${overlay.last_update}</span></span>
      </div>
    `;

    container.appendChild(card);
  });
}

// Tabs switching handler
function initTabs() {
  const navPackages = document.getElementById('nav-packages');
  const navOverlays = document.getElementById('nav-overlays');
  const navDocs = document.getElementById('nav-docs');

  const secPackages = document.getElementById('section-packages');
  const secOverlays = document.getElementById('section-overlays');
  const secDocs = document.getElementById('section-docs');

  function switchTab(tab) {
    currentTab = tab;
    
    // Manage navbar state
    [navPackages, navOverlays, navDocs].forEach(link => {
      if (link) link.classList.remove('active-nav-link');
    });

    // Manage section visibility
    [secPackages, secOverlays, secDocs].forEach(sec => {
      if (sec) sec.classList.add('hidden');
    });

    if (tab === 'packages') {
      if (navPackages) navPackages.classList.add('active-nav-link');
      if (secPackages) secPackages.classList.remove('hidden');
    } else if (tab === 'overlays') {
      if (navOverlays) navOverlays.classList.add('active-nav-link');
      if (secOverlays) secOverlays.classList.remove('hidden');
    } else if (tab === 'docs') {
      if (navDocs) navDocs.classList.add('active-nav-link');
      if (secDocs) secDocs.classList.remove('hidden');
    }
  }

  if (navPackages) navPackages.addEventListener('click', (e) => { e.preventDefault(); switchTab('packages'); });
  if (navOverlays) navOverlays.addEventListener('click', (e) => { e.preventDefault(); switchTab('overlays'); });
  if (navDocs) navDocs.addEventListener('click', (e) => { e.preventDefault(); switchTab('docs'); });
}

// Docs navigation sub-tabs
window.switchDoc = function(docId) {
  document.querySelectorAll('.doc-nav-item').forEach(item => {
    item.classList.remove('active-doc');
  });

  document.querySelectorAll('.doc-panel').forEach(panel => {
    panel.classList.add('hidden');
  });

  const activeLink = document.querySelector(`a[href="#doc-${docId === 'intro' ? 'introduction' : docId === 'install' ? 'installation' : docId}"]`);
  if (activeLink) activeLink.classList.add('active-doc');

  const activePanel = document.getElementById(`doc-panel-${docId}`);
  if (activePanel) activePanel.classList.remove('hidden');
}
