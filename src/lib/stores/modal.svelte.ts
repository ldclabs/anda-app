export const diagnosisModalState = $state({
  open: false,
  view: 'kip' as 'kip' | 'conversation'
})

export function openDiagnosisModal(view: 'kip' | 'conversation' = 'kip') {
  diagnosisModalState.view = view
  diagnosisModalState.open = true
}
