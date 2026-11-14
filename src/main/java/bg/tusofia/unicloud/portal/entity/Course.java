package bg.tusofia.unicloud.portal.entity;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.Table;

@Entity
@Table(name = "courses")
public class Course {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Integer id;

    @Column(nullable = false, unique = true, length = 16)
    private String code;

    @Column(nullable = false, length = 200)
    private String title;

    @Column(name = "ects_credits", nullable = false)
    private Integer ectsCredits;

    protected Course() {
    }

    public Course(String code, String title, Integer ectsCredits) {
        this.code = code;
        this.title = title;
        this.ectsCredits = ectsCredits;
    }

    public Integer getId() {
        return id;
    }

    public String getCode() {
        return code;
    }

    public void setCode(String code) {
        this.code = code;
    }

    public String getTitle() {
        return title;
    }

    public void setTitle(String title) {
        this.title = title;
    }

    public Integer getEctsCredits() {
        return ectsCredits;
    }

    public void setEctsCredits(Integer ectsCredits) {
        this.ectsCredits = ectsCredits;
    }
}
