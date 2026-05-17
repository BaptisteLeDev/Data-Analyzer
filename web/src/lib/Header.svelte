<header>
  <div class="brand">
    <svg class="logo" viewBox="0 0 64 64" aria-hidden="true">
      <rect width="64" height="64" fill="#000091"/>
      <rect x="11" y="42" width="8" height="14" fill="#FFFFFF"/>
      <rect x="22" y="32" width="8" height="24" fill="#FFFFFF"/>
      <rect x="33" y="22" width="8" height="34" fill="#FFFFFF"/>
      <rect x="44" y="10" width="8" height="46" fill="#E1000F"/>
      <rect x="11" y="56" width="41" height="2" fill="#FFFFFF"/>
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
    width: 44px;
    height: 44px;
    display: block;
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
