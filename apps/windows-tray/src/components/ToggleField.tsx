type Props = {
  label: string
  checked: boolean
  onChange: (value: boolean) => void
}

export function ToggleField({ label, checked, onChange }: Props) {
  return (
    <label className="toggle-row">
      <span>{label}</span>
      <button
        type="button"
        className={`toggle ${checked ? 'on' : 'off'}`}
        aria-pressed={checked}
        onClick={() => onChange(!checked)}
      >
        <span />
      </button>
    </label>
  )
}
