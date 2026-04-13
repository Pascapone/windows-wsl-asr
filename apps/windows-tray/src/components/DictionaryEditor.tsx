type Props = {
  value: string
  onChange: (value: string) => void
}

export function DictionaryEditor({ value, onChange }: Props) {
  return (
    <label className="dictionary-editor">
      <span>Ein Begriff pro Zeile</span>
      <textarea
        value={value}
        onChange={(event) => onChange(event.target.value)}
        rows={10}
        placeholder={'Pibo\nOpenClaw\nPascal'}
      />
    </label>
  )
}
