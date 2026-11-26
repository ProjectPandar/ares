use std::io::{self, BufWriter, Write};
use std::path::Path;

use ares_core::{
    CliPlatesRequest, CliPlatesSettings, CliProjectSliceRequest, CliUpwardMachine, ConfigOverrides,
    DirectProjectSliceRequest, GCODE_OUTPUT_BLOCK_SIZE, GCodeSink, GenerationMetadata, IoWriteSink,
    SliceError, SlicedPlate, SlicerContext, UpwardMachineInput, fn_sink,
    slice_cli_project_plate_validated, slice_cli_project_plate_validated_to_sink,
    slice_cli_project_plates, slice_cli_project_plates_to_sink, slice_project_plate,
    slice_project_plate_processed, slice_project_plate_processed_to_sink,
    slice_project_plate_to_sink, slice_project_plate_validated,
    slice_project_plate_validated_to_sink,
};

fn fixture(rel: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read(&path).unwrap_or_else(|err| panic!("failed reading {}: {err}", path.display()))
}

fn direct_req<'a>(
    plate: Option<u32>,
    overrides: &'a ConfigOverrides,
) -> DirectProjectSliceRequest<'a> {
    DirectProjectSliceRequest {
        plate,
        overrides,
        system_catalog: None,
    }
}

struct RecordingSink {
    bytes: Vec<u8>,
    chunk_sizes: Vec<usize>,
    flushes: usize,
}

impl RecordingSink {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            chunk_sizes: Vec::new(),
            flushes: 0,
        }
    }
}

impl GCodeSink for RecordingSink {
    fn write_chunk(&mut self, chunk: &[u8]) -> Result<(), SliceError> {
        assert!(!chunk.is_empty(), "sink should never receive empty chunks");
        self.chunk_sizes.push(chunk.len());
        self.bytes.extend_from_slice(chunk);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SliceError> {
        self.flushes += 1;
        Ok(())
    }
}

#[tokio::test]
async fn streaming_sink_matches_vec_output_byte_for_byte_across_multiple_chunks() {
    let overrides = ConfigOverrides::default();
    let meta = GenerationMetadata::default();
    for path in [
        "tests/parity/validation/bbl-toolchange-boundary.3mf",
        "tests/parity/arachne/default-prisms.3mf",
    ] {
        let project = fixture(path);
        let expected = slice_project_plate(&project, meta, None, &overrides, SlicerContext::new(0))
            .await
            .expect("buffered slice succeeds");

        let mut recorded = RecordingSink::new();
        let streamed = slice_project_plate_to_sink(
            &project,
            direct_req(None, &overrides),
            meta,
            SlicerContext::new(0),
            &mut recorded,
        )
        .await
        .expect("recording sink succeeds");
        assert!(
            recorded.chunk_sizes.len() > 1,
            "{path}: {:?}",
            recorded.chunk_sizes
        );
        assert_eq!(recorded.flushes, 1);
        assert_eq!(streamed.bytes_written, expected.len());
        assert_eq!(recorded.bytes, expected);

        let mut vec_sink = Vec::new();
        let streamed_vec = slice_project_plate_to_sink(
            &project,
            direct_req(None, &overrides),
            meta,
            SlicerContext::new(0),
            &mut vec_sink,
        )
        .await
        .expect("Vec<u8> sink succeeds");
        assert_eq!(streamed_vec.bytes_written, expected.len());
        assert_eq!(vec_sink, expected);

        let mut io_sink = IoWriteSink::new(BufWriter::with_capacity(
            GCODE_OUTPUT_BLOCK_SIZE,
            Vec::new(),
        ));
        let streamed_io = slice_project_plate_to_sink(
            &project,
            direct_req(None, &overrides),
            meta,
            SlicerContext::new(0),
            &mut io_sink,
        )
        .await
        .expect("IoWriteSink succeeds");
        assert_eq!(streamed_io.bytes_written, expected.len());
        assert_eq!(io_sink.bytes_written(), expected.len());
        assert!(io_sink.take_error().is_none());
        assert_eq!(io_sink.into_inner().into_inner().unwrap(), expected);
    }
}

#[tokio::test]
async fn streaming_processed_preserves_exact_moves_and_statistics() {
    let project = fixture("tests/parity/validation/bbl-toolchange-boundary.3mf");
    let overrides = ConfigOverrides::default();
    let meta = GenerationMetadata::default();
    let processed =
        slice_project_plate_processed(&project, meta, None, &overrides, SlicerContext::new(0))
            .await
            .expect("buffered processed slice succeeds");

    let mut chunks = 0usize;
    let mut collected = Vec::new();
    let mut closure = fn_sink(|chunk| {
        assert!(!chunk.is_empty());
        chunks += 1;
        collected.extend_from_slice(chunk);
        Ok(())
    });
    let streamed = slice_project_plate_processed_to_sink(
        &project,
        direct_req(None, &overrides),
        meta,
        SlicerContext::new(0),
        &mut closure,
    )
    .await
    .expect("streaming processed slice succeeds");

    assert!(chunks > 1);
    assert_eq!(streamed.bytes_written, processed.gcode.len());
    assert_eq!(collected, processed.gcode);
    assert_eq!(streamed.result, processed.result);
    assert_eq!(streamed.print_statistics, processed.print_statistics);
    assert_eq!(streamed.warnings, processed.warnings);
}

#[tokio::test]
async fn validated_and_cli_plate_streaming_match_buffered_apis() {
    let project = fixture("tests/parity/validation/bbl-toolchange-boundary.3mf");
    let overrides = ConfigOverrides::default();
    let meta = GenerationMetadata::default();

    let expected_val =
        slice_project_plate_validated(&project, meta, None, &overrides, SlicerContext::new(0))
            .await
            .expect("validated buffered slice succeeds");
    let mut val_sink = RecordingSink::new();
    let streamed_val = slice_project_plate_validated_to_sink(
        &project,
        direct_req(None, &overrides),
        meta,
        SlicerContext::new(0),
        &mut val_sink,
    )
    .await
    .expect("validated streaming slice succeeds");
    assert!(val_sink.chunk_sizes.len() > 1);
    assert_eq!(streamed_val.bytes_written, expected_val.len());
    assert_eq!(val_sink.bytes, expected_val);

    let (mut exp_upward, mut exp_plate) = (Vec::new(), SlicedPlate::default());
    let exp_cli = slice_cli_project_plate_validated(
        &project,
        CliProjectSliceRequest {
            plate: Some(1),
            overrides: &overrides,
            record_moves: true,
            orient: false,
            upward: CliUpwardMachine {
                source: None,
                input: UpwardMachineInput::FetchOriginal(None),
                report: &mut exp_upward,
            },
            sliced_plate: &mut exp_plate,
        },
        meta,
        SlicerContext::new(0),
    )
    .await
    .expect("cli buffered slice succeeds");

    let (mut str_upward, mut str_plate) = (Vec::new(), SlicedPlate::default());
    let mut cli_sink = RecordingSink::new();
    let str_cli = slice_cli_project_plate_validated_to_sink(
        &project,
        CliProjectSliceRequest {
            plate: Some(1),
            overrides: &overrides,
            record_moves: true,
            orient: false,
            upward: CliUpwardMachine {
                source: None,
                input: UpwardMachineInput::FetchOriginal(None),
                report: &mut str_upward,
            },
            sliced_plate: &mut str_plate,
        },
        meta,
        SlicerContext::new(0),
        &mut cli_sink,
    )
    .await
    .expect("cli streaming slice succeeds");

    assert!(cli_sink.chunk_sizes.len() > 1);
    assert_eq!(str_cli.bytes_written, exp_cli.gcode.len());
    assert_eq!(cli_sink.bytes, exp_cli.gcode);
    assert_eq!(str_cli.result, exp_cli.result);
    assert_eq!(str_cli.print_statistics, exp_cli.print_statistics);
    assert_eq!(str_upward, exp_upward);
    assert_eq!(str_plate, exp_plate);

    fn plates_req<'a>(
        overrides: &'a ConfigOverrides,
        upward: &'a mut Vec<String>,
    ) -> CliPlatesRequest<'a> {
        CliPlatesRequest {
            selection: 1,
            filename: "bbl-toolchange-boundary.3mf",
            settings: CliPlatesSettings::Embedded {
                system_catalog: None,
            },
            overrides,
            orient: false,
            upward: CliUpwardMachine {
                source: None,
                input: UpwardMachineInput::FetchOriginal(None),
                report: upward,
            },
        }
    }
    let (mut buf_plates, mut buf_up) = (Vec::new(), Vec::new());
    slice_cli_project_plates(
        &project,
        plates_req(&overrides, &mut buf_up),
        meta,
        SlicerContext::new(0),
        |p, g, r| {
            buf_plates.push((p, g.to_vec(), r.clone()));
            Ok(())
        },
    )
    .expect("buffered cli plates succeed");

    let (mut str_plates, mut str_up) = (Vec::new(), Vec::new());
    slice_cli_project_plates_to_sink(
        &project,
        plates_req(&overrides, &mut str_up),
        meta,
        SlicerContext::new(0),
        (
            |_, _| Ok(RecordingSink::new()),
            |plate, sink, report| {
                assert!(sink.chunk_sizes.len() > 1);
                str_plates.push((plate, sink.bytes, report.clone()));
                Ok(())
            },
        ),
    )
    .expect("streaming cli plates succeed");
    assert_eq!(str_plates, buf_plates);
    assert_eq!(str_up, buf_up);
}

#[tokio::test]
async fn streaming_sink_writes_zero_bytes_on_validation_and_slice_errors() {
    let project = fixture("tests/parity/validation/bbl-toolchange-boundary.3mf");
    let overrides = ConfigOverrides::default();
    let meta = GenerationMetadata::default();
    let mut sink = RecordingSink::new();

    for (input, plate, ovr) in [
        (&b""[..], None, overrides.clone()),
        (&b"PK\x03\x04corrupted"[..], None, overrides.clone()),
        (&project[..], Some(99), overrides.clone()),
        (
            &project[..],
            None,
            ConfigOverrides::from_json(br#"{"layer_height":"99.0"}"#).unwrap(),
        ),
    ] {
        let res = slice_project_plate_validated_to_sink(
            input,
            direct_req(plate, &ovr),
            meta,
            SlicerContext::new(0),
            &mut sink,
        )
        .await;
        assert!(res.is_err());
        assert!(sink.bytes.is_empty() && sink.chunk_sizes.is_empty());
    }
}

struct FailingWriter {
    fail_after_writes: usize,
    writes: usize,
}

impl Write for FailingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        if self.writes > self.fail_after_writes {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "simulated full disk",
            ));
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn streaming_sink_propagates_sink_and_io_errors() {
    let project = fixture("tests/parity/validation/bbl-toolchange-boundary.3mf");
    let overrides = ConfigOverrides::default();
    let meta = GenerationMetadata::default();

    let mut seen_chunks = 0usize;
    let mut failing_fn = fn_sink(|_| {
        seen_chunks += 1;
        if seen_chunks == 2 {
            return Err(SliceError::InvalidInput("sink aborted".to_owned()));
        }
        Ok(())
    });
    let err = slice_project_plate_to_sink(
        &project,
        direct_req(None, &overrides),
        meta,
        SlicerContext::new(0),
        &mut failing_fn,
    )
    .await
    .expect_err("failing closure sink must abort slicing");
    assert_eq!(seen_chunks, 2);
    assert!(matches!(err, SliceError::InvalidInput(msg) if msg == "sink aborted"));

    let mut io_sink = IoWriteSink::new(FailingWriter {
        fail_after_writes: 1,
        writes: 0,
    });
    let err = slice_project_plate_to_sink(
        &project,
        direct_req(None, &overrides),
        meta,
        SlicerContext::new(0),
        &mut io_sink,
    )
    .await
    .expect_err("failing IoWriteSink must return error");
    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert_eq!(
        io_sink.take_error().expect("original io::Error").kind(),
        io::ErrorKind::WriteZero
    );
}
