package bg.tusofia.unicloud.portal.controller;

import java.net.URI;
import java.util.List;

import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

import bg.tusofia.unicloud.portal.dto.OfferingRequest;
import bg.tusofia.unicloud.portal.dto.OfferingResponse;
import bg.tusofia.unicloud.portal.service.CourseOfferingService;
import jakarta.validation.Valid;

@RestController
@RequestMapping("/api/v1/offerings")
public class CourseOfferingController {

    private final CourseOfferingService offeringService;

    public CourseOfferingController(CourseOfferingService offeringService) {
        this.offeringService = offeringService;
    }

    @GetMapping
    public List<OfferingResponse> getAll(
            @RequestParam(required = false) String semester,
            @RequestParam(required = false) Integer courseId,
            @RequestParam(required = false) Boolean open) {
        return offeringService.findAll(semester, courseId, open);
    }

    @GetMapping("/{id}")
    public OfferingResponse getById(@PathVariable Long id) {
        return offeringService.findById(id);
    }

    @PostMapping
    public ResponseEntity<OfferingResponse> create(@Valid @RequestBody OfferingRequest request) {
        OfferingResponse created = offeringService.create(request);
        URI location = URI.create("/api/v1/offerings/" + created.id());
        return ResponseEntity.created(location).body(created);
    }

    @PutMapping("/{id}")
    public OfferingResponse update(@PathVariable Long id, @Valid @RequestBody OfferingRequest request) {
        return offeringService.update(id, request);
    }
}
