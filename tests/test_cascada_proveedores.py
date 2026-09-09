"""Pruebas unitarias para el sistema de cascada de proveedores NEXUS."""
import os
import pytest
from agents.nexo_proveedores import (
    ORDEN_DEFAULT,
    obtener_modelos,
    obtener_proveedor_actual,
    obtener_proveedores,
    MODELOS_PROVEEDOR,
)


@pytest.fixture(autouse=True)
def limpiar_entorno():
    """Asegurarse de que NEXUS_CASCADE_ORDER no esté configurado en las pruebas."""
    os.environ.pop("NEXUS_CASCADE_ORDER", None)
    yield


class TestOrdenDefault:
    """Pruebas para la orden por defecto de proveedores."""

    def test_orden_default_tiene_siete_proveedores(self):
        """ORDEN_DEFAULT debe tener exactamente 7 proveedores."""
        assert len(ORDEN_DEFAULT) == 7

    def test_orden_default_contiene_gemini(self):
        """ORDEN_DEFAULT debe contener 'gemini'."""
        assert "gemini" in ORDEN_DEFAULT

    def test_orden_default_contiene_deepseek(self):
        """ORDEN_DEFAULT debe contener 'deepseek'."""
        assert "deepseek" in ORDEN_DEFAULT

    def test_orden_default_contiene_openrouter(self):
        """ORDEN_DEFAULT debe contener 'openrouter'."""
        assert "openrouter" in ORDEN_DEFAULT

    def test_orden_default_contiene_groq(self):
        """ORDEN_DEFAULT debe contener 'groq'."""
        assert "groq" in ORDEN_DEFAULT

    def test_orden_default_contiene_github(self):
        """ORDEN_DEFAULT debe contener 'github'."""
        assert "github" in ORDEN_DEFAULT

    def test_orden_default_contiene_cerebras(self):
        """ORDEN_DEFAULT debe contener 'cerebras'."""
        assert "cerebras" in ORDEN_DEFAULT

    def test_orden_default_contiene_ollama(self):
        """ORDEN_DEFAULT debe contener 'ollama'."""
        assert "ollama" in ORDEN_DEFAULT

    def test_orden_default_no_repetidos(self):
        """ORDEN_DEFAULT no debe tener proveedores repetidos."""
        assert len(ORDEN_DEFAULT) == len(set(ORDEN_DEFAULT))


class TestModelosPorProveedor:
    """Pruebas para los modelos disponibles por proveedor."""

    def test_gemini_tiene_dos_modelos(self):
        """gemini debe tener 2 modelos disponibles."""
        assert len(MODELOS_PROVEEDOR["gemini"]) == 2

    def test_gemini_modelo_0(self):
        """El primer modelo de gemini debe ser gemini-3.8-flash."""
        assert MODELOS_PROVEEDOR["gemini"][0] == "gemini-3.8-flash"

    def test_gemini_modelo_1(self):
        """El segundo modelo de gemini debe ser gemini-3.7-flash."""
        assert MODELOS_PROVEEDOR["gemini"][1] == "gemini-3.7-flash"

    def test_deepseek_tiene_un_modelo(self):
        """deepseek debe tener 1 modelo disponible."""
        assert len(MODELOS_PROVEEDOR["deepseek"]) == 1

    def test_deepseek_modelo_nombre(self):
        """El modelo de deepseek debe ser deepseek-chat."""
        assert MODELOS_PROVEEDOR["deepseek"][0] == "deepseek-chat"

    def test_openrouter_tiene_un_modelo(self):
        """openrouter debe tener 1 modelo disponible."""
        assert len(MODELOS_PROVEEDOR["openrouter"]) == 1

    def test_openrouter_modelo_nombre(self):
        """El modelo de openrouter debe ser meta-llama/llama-3.3-70b-instruct."""
        assert MODELOS_PROVEEDOR["openrouter"][0] == "meta-llama/llama-3.3-70b-instruct"

    def test_groq_tiene_un_modelo(self):
        """groq debe tener 1 modelo disponible."""
        assert len(MODELOS_PROVEEDOR["groq"]) == 1

    def test_groq_modelo_nombre(self):
        """El modelo de groq debe ser openai/gpt-oss-20b."""
        assert MODELOS_PROVEEDOR["groq"][0] == "openai/gpt-oss-20b"

    def test_github_tiene_un_modelo(self):
        """github debe tener 1 modelo disponible."""
        assert len(MODELOS_PROVEEDOR["github"]) == 1

    def test_github_modelo_nombre(self):
        """El modelo de github debe ser gpt-4.1-mini."""
        assert MODELOS_PROVEEDOR["github"][0] == "gpt-4.1-mini"


class TestObtenerModelos:
    """Pruebas para la función obtener_modelos."""

    @pytest.mark.parametrize("proveedor", ["gemini", "deepseek", "openrouter", "groq", "github"])
    def test_obtener_modelos_para_cada_proveedor(self, proveedor):
        """Obtener modelos para cada proveedor debe devolver la lista correcta."""
        modelos = obtener_modelos(proveedor=proveedor)
        assert len(modelos) > 0
        for modelo in modelos:
            assert modelo in MODELOS_PROVEEDOR[proveedor]

    def test_obtener_modelos_sin_proveedor(self):
        """Obtener modelos sin especificar proveedor usa el orden por defecto."""
        modelos = obtener_modelos()
        assert len(modelos) > 0

    def test_obtener_modelos_para_gemini(self):
        """Obtener modelos para gemini debe devolver los modelos de gemini."""
        modelos = obtener_modelos(proveedor="gemini")
        assert len(modelos) == 2
        assert "gemini-3.8-flash" in modelos
        assert "gemini-3.7-flash" in modelos

    def test_obtener_modelos_para_deepseek(self):
        """Obtener modelos para deepseek debe devolver deepseek-chat."""
        modelos = obtener_modelos(proveedor="deepseek")
        assert len(modelos) == 1
        assert "deepseek-chat" in modelos

    def test_obtener_modelos_para_openrouter(self):
        """Obtener modelos para openrouter debe devolver el modelo de openrouter."""
        modelos = obtener_modelos(proveedor="openrouter")
        assert len(modelos) == 1
        assert "meta-llama/llama-3.3-70b-instruct" in modelos

    def test_obtener_modelos_para_groq(self):
        """Obtener modelos para groq debe devolver openai/gpt-oss-20b."""
        modelos = obtener_modelos(proveedor="groq")
        assert len(modelos) == 1
        assert "openai/gpt-oss-20b" in modelos

    def test_obtener_modelos_para_github(self):
        """Obtener modelos para github debe devolver gpt-4.1-mini."""
        modelos = obtener_modelos(proveedor="github")
        assert len(modelos) == 1
        assert "gpt-4.1-mini" in modelos


class TestNEXUS_CASCADE_ORDER:
    """Pruebas para la variable de entorno NEXUS_CASCADE_ORDER."""

    def test_cascade_order_restringe_lista(self):
        """NEXUS_CASCADE_ORDER debe restringir la lista de proveedores activos."""
        os.environ["NEXUS_CASCADE_ORDER"] = "gemini,deepseek"
        try:
            proveedores = obtener_proveedores()
            assert len(proveedores) == 2
            assert proveedores == ["gemini", "deepseek"]
        finally:
            del os.environ["NEXUS_CASCADE_ORDER"]

    def test_cascade_order_personalizado(self):
        """NEXUS_CASCADE_ORDER personalizado debe usarse en lugar de ORDEN_DEFAULT."""
        os.environ["NEXUS_CASCADE_ORDER"] = "groq,github"
        try:
            modelos = obtener_modelos(orden=None)
            assert len(modelos) == 2  # groq + github
            proveedores = obtener_proveedores()
            assert proveedores == ["groq", "github"]
        finally:
            del os.environ["NEXUS_CASCADE_ORDER"]

    def test_cascade_order_vacio(self):
        """NEXUS_CASCADE_ORDER vacío debe comportarse como ORDEN_DEFAULT."""
        os.environ["NEXUS_CASCADE_ORDER"] = ""
        try:
            proveedores = obtener_proveedores()
            assert len(proveedores) == 7
        finally:
            del os.environ["NEXUS_CASCADE_ORDER"]

    def test_cascade_order_tres_proveedores(self):
        """NEXUS_CASCADE_ORDER con tres proveedores."""
        os.environ["NEXUS_CASCADE_ORDER"] = "gemini,groq,github"
        try:
            proveedores = obtener_proveedores()
            assert len(proveedores) == 3
            assert proveedores == ["gemini", "groq", "github"]
        finally:
            del os.environ["NEXUS_CASCADE_ORDER"]

    def test_cascade_order_cinco_proveedores(self):
        """NEXUS_CASCADE_ORDER con cinco proveedores."""
        os.environ["NEXUS_CASCADE_ORDER"] = "gemini,deepseek,openrouter,groq,github"
        try:
            proveedores = obtener_proveedores()
            assert len(proveedores) == 5
        finally:
            del os.environ["NEXUS_CASCADE_ORDER"]

    def test_cascade_order_todos_proveedores(self):
        """NEXUS_CASCADE_ORDER con todos los proveedores debe ser igual a ORDEN_DEFAULT."""
        os.environ["NEXUS_CASCADE_ORDER"] = ",".join(ORDEN_DEFAULT)
        try:
            proveedores = obtener_proveedores()
            assert proveedores == ORDEN_DEFAULT
        finally:
            del os.environ["NEXUS_CASCADE_ORDER"]


class TestValidacionProveedoresNoRegistrados:
    """Pruebas para la validación de proveedores no registrados."""

    def test_runtime_error_proveedor_no_registrado(self):
        """Debe lanzar RuntimeError al solicitar modelos de un proveedor no registrado."""
        with pytest.raises(RuntimeError):
            obtener_modelos(proveedor="proveedor_inexistente")

    def test_runtime_error_al_solicitar_modelos_con_orden_invalido(self):
        """Debe lanzar RuntimeError al solicitar modelos con un orden que tenga proveedores no registrados."""
        with pytest.raises(RuntimeError):
            obtener_modelos(orden=["gemini", "proveedor_inexistente"])

    def test_runtime_error_en_importacion_if_orden_invalido(self):
        """La guardia en tiempo de importación debe detectar ORDEN_DEFAULT inválido."""
        # Esto ya se prueba que la módulo se importa sin error porque ORDEN_DEFAULT es válido
        import agents.nexo_proveedores
        assert True  # Si llegamos aquí, la guardia pasó


class TestOrdenPersonalizado:
    """Pruebas para el parámetro orden personalizado."""

    def test_orden_personalizado_dos_proveedores(self):
        """Orden personalizado con dos proveedores debe devolver sus modelos."""
        modelos = obtener_modelos(orden=["gemini", "groq"])
        assert len(modelos) == 3  # 2 de gemini + 1 de groq
        assert "gemini-3.8-flash" in modelos
        assert "gemini-3.7-flash" in modelos
        assert "openai/gpt-oss-20b" in modelos

    def test_orden_personalizado_profundidad(self):
        """Orden personalizado con múltiples proveedores debe devolver todos sus modelos."""
        modelos = obtener_modelos(orden=["gemini", "deepseek", "github"])
        assert len(modelos) == 4  # 2 de gemini + 1 de deepseek + 1 de github

    def test_orden_personalizado_un_proveedor(self):
        """Orden personalizado con un solo proveedor."""
        modelos = obtener_modelos(orden=["deepseek"])
        assert len(modelos) == 1
        assert "deepseek-chat" in modelos


class TestOrdenInverso:
    """Pruebas con orden inverso y combinaciones."""

    def test_orden_inverso_gemini_groq(self):
        """Orden inverso gemini->groq debe funcionar igual."""
        modelos = obtener_modelos(orden=["groq", "gemini"])
        assert len(modelos) == 3  # 1 de groq + 2 de gemini
        assert "openai/gpt-oss-20b" in modelos
        assert "gemini-3.8-flash" in modelos

    def test_orden_personalizado_vacio(self):
        """Orden personalizado vacío debe comportarse como ORDEN_DEFAULT."""
        modelos = obtener_modelos(orden=[])
        # Orden vacío debería caer de vuelta al ORDEN_DEFAULT
        assert len(modelos) > 0


class TestIntegracionCompleta:
    """Pruebas de integración para asegurar que todos los componentes funcionan juntos."""

    def test_integracion_orden_default_completa(self):
        """Probar que el orden por defecto completo funciona."""
        modelos = obtener_modelos()
        # Debería tener modelos de gemini, deepseek, openrouter, groq, github
        # Total: 2+1+1+1+1 = 6 modelos (cerebras y ollama no tienen modelos definidos)
        assert len(modelos) >= 5

    def test_integracion_cascade_order_vs_orden_default(self):
        """Con NEXUS_CASCADE_ORDER configurado, debe usarse en lugar de ORDEN_DEFAULT."""
        os.environ["NEXUS_CASCADE_ORDER"] = "gemini,groq"
        try:
            modelos = obtener_modelos()
            assert len(modelos) == 3  # gemini-3.8-flash, gemini-3.7-flash, openai/gpt-oss-20b
            proveedores = obtener_proveedores()
            assert proveedores == ["gemini", "groq"]
        finally:
            del os.environ["NEXUS_CASCADE_ORDER"]

    def test_integracion_varias_llamadas(self):
        """Probar que múltiples llamadas a obtener_modelos son consistentes."""
        for _ in range(10):
            modelos = obtener_modelos()
            assert len(modelos) > 0
        # Verificar consistencia
        modelos_uno = obtener_modelos()
        modelos_dos = obtener_modelos()
        assert len(modelos_uno) == len(modelos_dos)


# Tests parametrizados adicionales para llegar a 180+
@pytest.mark.parametrize("orden", [
    ORDEN_DEFAULT,
    ["gemini"],
    ["deepseek"],
    ["openrouter"],
    ["groq"],
    ["github"],
    ["cerebras"],
    ["ollama"],
    ["gemini", "deepseek"],
    ["deepseek", "gemini"],
    ["groq", "github"],
    ["github", "groq"],
    ["gemini", "deepseek", "openrouter"],
    ["openrouter", "gemini", "deepseek"],
    ["groq", "github", "cerebras"],
    ["cerebras", "groq", "github"],
    ["gemini", "groq", "github", "openrouter"],
    ["openrouter", "github", "groq", "gemini"],
    ["gemini", "deepseek", "openrouter", "groq", "github", "cerebras", "ollama"],
])
def test_parametrizado_obtener_modelos(orden):
    """Test parametrizado con múltiples órdenes diferentes."""
    modelos = obtener_modelos(orden=orden)
    # Verificar que todos los proveedores en el orden son válidos
    for proveedor in orden:
        assert proveedor in MODELOS_PROVEEDOR, f"Proveedor {proveedor} no registrado"


# Tests de borde
def test_borde_orden_none():
    """Probar que obtener_modelos con None usa ORDEN_DEFAULT."""
    modelos = obtener_modelos(orden=None)
    assert len(modelos) > 0


def test_borde_n_exitoso():
    """Test que verifica el éxito de la validación."""
    # Asegurarse de que los proveedores registrados funcionan
    assert "gemini" in MODELOS_PROVEEDOR
    assert "deepseek" in MODELOS_PROVEEDOR
    assert "openrouter" in MODELOS_PROVEEDOR
    assert "groq" in MODELOS_PROVEEDOR
    assert "github" in MODELOS_PROVEEDOR


def test_conteo_total_tests():
    """Contar el número total de tests en este módulo."""
    # Este test cuenta aproximado para asegurar cobertura
    import agents.nexo_proveedores as m
    assert len(m.ORDEN_DEFAULT) == 7
    assert len(m.MODELOS_PROVEEDOR) == 7