import { useStore } from "../../store";
import { ScanProgress } from "../ScanProgress/ScanProgress";
import { ResultsToolbar } from "./ResultsToolbar";
import { ResultsTree } from "./ResultsTree";
import { EmptyState } from "../EmptyState/EmptyState";
import styles from "./ResultsView.module.css";

export function ResultsView() {
  const { scanProgress, isScanning } = useStore();

  if (!scanProgress && !isScanning) {
    return <EmptyState />;
  }

  return (
    <div className={styles.container}>
      <ScanProgress />
      {(scanProgress || isScanning) && (
        <>
          <ResultsToolbar />
          <ResultsTree />
        </>
      )}
    </div>
  );
}
