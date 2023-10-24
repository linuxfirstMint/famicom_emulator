from gen_opcodes import generate_rust_code, set_target_path


def test_set_target_path():
    path = "src"
    filename = "opcodes.rs"
    output_path = set_target_path(path, filename)
    assert output_path.name == filename
    assert output_path.parent.name == path


def test_set_target_path_with_custom_values():
    test_path = "tool"
    output_path = set_target_path(test_path, "custom_filename.rs")
    assert output_path.name == "custom_filename.rs"
    assert output_path.parent.name == test_path


def test_set_target_path_with_nonexistent_path():
    try:
        set_target_path("nonexistent_path")
    except FileNotFoundError:
        assert True
    else:
        assert False
