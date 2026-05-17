<header>
  <div class="brand">
    <svg class="logo" viewBox="0 0 48 48" aria-hidden="true">
      <rect x="0" y="0" width="16" height="48" fill="#000091" />
      <rect x="16" y="0" width="16" height="48" fill="#FFFFFF" />
      <rect x="32" y="0" width="16" height="48" fill="#E1000F" />
      <text x="24" y="32" text-anchor="middle" font-family="Marianne, Inter, sans-serif"
            font-size="20" font-weight="700" fill="#161616">DA</text>
    </svg>
    <div class="titles">
      <span class="name">DATA-ANALYZER</span>
      <span class="sub">prénoms rares · état civil INSEE 2006</span>
    </div>
  </div>
</header>

<style>
  header {
    border-bottom: 1px solid var(--border);
    background: white;
    padding: calc(var(--space) * 2) calc(var(--space) * 3);
  }
  .brand { display: flex; align-items: center; gap: calc(var(--space) * 2); }
  .logo {
    width: 40px;
    height: 40px;
    border: 1px solid var(--border);
  }
  .titles { display: flex; flex-direction: column; line-height: 1.2; }
  .name {
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--bleu-rep);
  }
  .sub {
    font-size: 0.85rem;
    color: var(--text-mute);
  }
</style>
