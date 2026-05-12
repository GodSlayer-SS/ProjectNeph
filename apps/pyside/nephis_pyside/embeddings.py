"""Phase 2 stub — sentence-transformers embeddings via bge-small-en-v1.5."""

import logging

log = logging.getLogger("nephis_pyside.embeddings")

MODEL_NAME = "BAAI/bge-small-en-v1.5"
DIM = 384


class SidecarEmbedder:
    """Encodes texts to 384-dim vectors. Phase 2 — stub returns zeros in Phase 1."""

    def __init__(self):
        self._model = None
        try:
            from sentence_transformers import SentenceTransformer
            self._model = SentenceTransformer(MODEL_NAME)
            log.info("SidecarEmbedder loaded: %s", MODEL_NAME)
        except Exception as e:
            log.warning("sentence-transformers unavailable: %s — returning zero vectors.", e)

    def encode(self, texts: list[str]) -> list[list[float]]:
        if self._model is not None:
            try:
                vectors = self._model.encode(texts, normalize_embeddings=True)
                return [v.tolist() for v in vectors]
            except Exception as e:
                log.error("Encode failed: %s", e)
        # Phase 1 stub — zeros
        return [[0.0] * DIM for _ in texts]
